<template>
	<div class = 'downloads'>
		<div class = 'content no-scrollbar'>
			<p v-if = 'state.error' role = 'alert'>{{ state.error }}</p>
			<p v-if = 'state.loading'>正在读取缓存…</p>
			<p v-else-if = '!state.items.length'>暂无缓存，可在播放页面缓存当前剧集。</p>
			<div v-if = '!seriesId' class = 'series-list'>
				<card v-for = 'group in groups' :key = 'group.id' :item = 'seriesCard(group)' :navigate = 'false' @select = 'openSeries(group)'/>
			</div>
			<section v-else-if = 'selectedGroup'>
				<var-cell v-for = 'item in selectedGroup.items' :key = 'item.id' :title = 'item.episodeTitle' border ripple @click = 'selectItem(item)'>
					<template #description>
						<div class = 'status'>{{ downloadStatus(item) }}</div>
						<progress v-if = 'item.status === "downloading" || item.status === "merging"' :value = 'progressValue(item)' :max = '1'/>
					</template>
					<template #extra>
						<div class = 'actions'>
							<var-button v-if = 'item.status === "failed"' size = 'small' :disabled = 'state.busy' @click.stop = 'retry(item)'>重试</var-button>
							<var-button size = 'small' type = 'danger' :disabled = 'state.busy' @click.stop = 'remove(item)'>{{ ['queued', 'downloading', 'merging'].includes(item.status) ? '取消并删除' : '删除' }}</var-button>
						</div>
					</template>
				</var-cell>
			</section>
			<var-button v-else-if = '!state.loading' text @click = 'backToSeries'>返回系列列表</var-button>
		</div>
	</div>
</template>
<script setup lang = 'ts'>
	import { computed, watch, onMounted, onUnmounted, reactive, type ComputedRef } from 'vue';
	import { useTravel } from '@/script/travel';
	import { Dialog, Snackbar } from '@varlet/ui';
	import Card from '@/ui/card.vue';
	import { type Item } from '@/script/invoke';
	import { deleteDownload, downloadDirectory, downloadStatus, listDownloads, startDownload, type Download, type DownloadHeader } from '@/script/downloads';

	interface Group { id : string; title : string; coverUrl : string; items : Download[]; }
	interface CacheState {
		items : Download[]; directory : string; loading : boolean; busy : boolean; error : string;
	}
	const { query, to_cache, to_cached_video } = useTravel();
	const emit : (event : 'header', header : DownloadHeader) => void = defineEmits<{ header : [header : DownloadHeader]; }>();
	const state : CacheState = reactive<CacheState>({ items: [], directory: '', loading: true, busy: false, error: '' });
	const groups : ComputedRef<Group[]> = computed(() : Group[] => {
		const grouped : Map<string, Group> = new Map<string, Group>();
		for (const item of state.items) {
			let group : Group | undefined = grouped.get(item.animeId);
			if (!group) {
				group = { id: item.animeId, title: item.animeTitle, coverUrl: item.coverUrl ?? '', items: [] };
				grouped.set(item.animeId, group);
			}
			group.items.push(item);
			group.coverUrl ||= item.coverUrl ?? '';
		}
		return [...grouped.values()];
	});
	const seriesId : ComputedRef<string> = computed(() : string => query.value.series);
	const selectedGroup : ComputedRef<Group | undefined> = computed(() : Group | undefined => groups.value.find((group : Group) : boolean => group.id === seriesId.value));
	const seriesCard = (group : Group) : Item => ({
		name: group.title, url: group.id, img: group.coverUrl,
		count: `已缓存 ${group.items.filter((item : Download) : boolean => item.status === 'completed').length} 集 · 共 ${group.items.length} 集`
	});
	const openSeries = async (group : Group) : Promise<void> => {
		await to_cache(group.id);
	};
	const backToSeries = async () : Promise<void> => {
		await to_cache('', true);
	};
	let timer : ReturnType<typeof setTimeout> | undefined;
	let disposed : boolean = false;
	const refresh = async () : Promise<void> => {
		try {
			const items : Download[] = await listDownloads();
			if (!disposed) {
				state.items = items; state.error = '';
				if (seriesId.value && !items.some((item : Download) : boolean => item.animeId === seriesId.value)) await backToSeries();
			}
		} catch (error : unknown) { if (!disposed) state.error = String(error); }
		finally { state.loading = false; }
	};
	const poll = async () : Promise<void> => {
		await refresh();
		if (!disposed) timer = setTimeout(() : void => { void poll(); }, 1000);
	};
	const play = async (item : Download) : Promise<void> => {
		try {
			await to_cached_video(item);
		} catch (error : unknown) { Snackbar.error(String(error)); }
	};
	const selectItem = async (item : Download) : Promise<void> => {
		if (state.busy) return;
		if (item.status === 'completed') await play(item);
		else Snackbar.info(downloadStatus(item));
	};
	const removeItems = async (items : Download[]) : Promise<void> => {
		state.busy = true;
		try {
			for (const item of items) await deleteDownload(item.id);
		} catch (error : unknown) { Snackbar.error(String(error)); }
		finally { await refresh(); state.busy = false; }
	};
	const remove = async (item : Download) : Promise<void> => {
		const action : string = await Dialog({ title: '删除缓存', message: `删除“${item.episodeTitle}”？下载中的临时文件也会一并清理。` });
		if (action === 'confirm') await removeItems([item]);
	};
	const removeGroup = async (group : Group) : Promise<void> => {
		const action : string = await Dialog({ title: '删除整部缓存', message: `删除“${group.title}”的全部 ${group.items.length} 集缓存和下载任务？` });
		if (action === 'confirm') await removeItems(group.items);
	};
	watch(() : readonly [Group | undefined, boolean] => [selectedGroup.value, state.busy],
		([group, busy] : readonly [Group | undefined, boolean]) : void => {
			emit('header', {
				seriesId: group?.id ?? '', title: group?.title ?? '', busy,
				remove: group ? () : Promise<void> => removeGroup(group) : undefined
			});
		}, { immediate: true });
	const retry = async (item : Download) : Promise<void> => {
		state.busy = true;
		try { await startDownload({ pageUrl: item.pageUrl, animeTitle: item.animeTitle, episodeTitle: item.episodeTitle, coverUrl: item.coverUrl }); }
		catch (error : unknown) { Snackbar.error(String(error)); }
		finally { await refresh(); state.busy = false; }
	};
	const progressValue = (item : Download) : number | undefined => {
		if (item.status === 'merging') return item.mergeProgress === null ? undefined : item.mergeProgress / 100;
		return item.totalSegments > 0 ? item.completedSegments / item.totalSegments
			: item.totalBytes ? item.downloadedBytes / item.totalBytes : undefined;
	};
	onMounted(async () : Promise<void> => {
		void poll();
		try { state.directory = await downloadDirectory(); }
		catch (error : unknown) { state.error = String(error); }
	});
	onUnmounted(() : void => { disposed = true; clearTimeout(timer); });
</script>
<style scoped lang = 'scss'>
	.downloads { height: 100%; }
	.content { height: 100%; overflow-y: auto; padding: 0 12px 20px; box-sizing: border-box; }
	.series-list { display: flex; flex-direction: column; gap: 10px; padding-top: 10px; }
	:deep(.var-cell__title) { white-space: normal; overflow-wrap: anywhere; }
	.status, .directory { font-size: 12px; overflow-wrap: anywhere; }
	.actions { display: flex; flex-direction: column; gap: 8px; }
	progress { width: 100%; accent-color: var(--color-primary); }
</style>
