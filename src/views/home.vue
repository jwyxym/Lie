<template>
	<main>
		<var-app-bar
			:title = 'appBarTitle'
			title-position = 'left'
		>
			<template v-if = 'page.ct === 3 && query.series' #left>
				<var-button text round @click = 'backToCacheSeries'>
					<var-icon name = 'chevron-left' :size = '24'/>
				</var-button>
			</template>
			<template #right>
				<var-icon v-if = 'page.ct === 3 && cacheHeader.seriesId === query.series && cacheHeader.remove'
					class = 'delete-cache-icon' name = 'trash-can-outline' :aria-disabled = 'cacheHeader.busy'
					@click = '!cacheHeader.busy && cacheHeader.remove()'/>
				<var-icon 
					namespace = 'i'
					:name = 'themes.name'
					:transition = '300'
					@click = 'themes.change()'
				/>
			</template>
		</var-app-bar>
		<TransitionGroup
			name = 'move'
			tag = 'div'
		>
			<browse v-if = 'page.ct == 1'/>
			<search v-else-if = 'page.ct == 2' v-model = 'page.search'/>
			<downloads v-else-if = 'page.ct == 3' @header = 'updateCacheHeader'/>
			<schedule v-else :list = 'page.schedule.list' v-model = 'page.day'/>
		</TransitionGroup>
		<var-tabs
			v-model:active = 'page.ct'
		>
			<var-tab>追番周表</var-tab>
			<var-tab>全部番剧</var-tab>
			<var-tab>搜素番剧</var-tab>
			<var-tab>视频缓存</var-tab>
		</var-tabs>
	</main>
</template>
<script setup lang = 'ts'>
	import { computed, watch, onMounted, onUnmounted, reactive, type ComputedRef } from 'vue';
	import { onBackButtonPress } from '@tauri-apps/api/app';
	import { exit } from '@tauri-apps/plugin-process';
	import { Snackbar } from '@varlet/ui';

	import schedule from './schedule.vue';
	import browse from './browse.vue';
	import search from './search.vue';
	import downloads from './downloads.vue';

	import { get_schedule, type Schedule } from '@/script/invoke';
	import { type DownloadHeader } from '@/script/downloads';
	import themes from '@/script/themes';
	import { useTravel } from '@/script/travel';

	const { query, to_home, to_cache } = useTravel();
	const cacheHeader : DownloadHeader = reactive({ seriesId: '', title: '', busy: false });
	const updateCacheHeader = (header : DownloadHeader) : void => { Object.assign(cacheHeader, header); };
	const backToCacheSeries = () : void => {
		void to_cache('', true);
	};
	const page = reactive({
		ct : computed({
			get : () : number => query.value.home,
			set : (i : number) : void => {
				void to_home(i);
			}
		}),
		day : computed({
			get : () : number => query.value.day,
			set : (i : number) : void => {
				void to_home(0, i);
			}
		}),
		search : computed({
			get : () : string => query.value.search,
			set : (i : string) : void => {
				void to_home(2, page.day, i);
			}
		}),
		schedule : {
			list : {} as Schedule,
			loaded : false
		}
	});
	const appBarTitle : ComputedRef<string> = computed(() : string => {
		if (page.ct === 1) return '全部番剧';
		if (page.ct === 2) return '搜素番剧';
		if (page.ct === 3) return cacheHeader.seriesId === query.value.series && cacheHeader.title ? cacheHeader.title : '视频缓存';
		return '追番周表';
	});

	watch(() : boolean => page.ct === 0, async (active : boolean) : Promise<void> => {
		if (active && !page.schedule.loaded) {
			page.schedule.loaded = true;
			page.schedule.list = await get_schedule();
		}
	}, { immediate : true });

	let listener: Awaited<ReturnType<typeof onBackButtonPress>> | undefined;
	let count = 0;
	onMounted(async () => {
		listener = await onBackButtonPress(() => {
			if (page.ct === 3 && query.value.series) {
				backToCacheSeries();
				return;
			}
			if (count)
				exit(1);
			else {
				count++;
				Snackbar.info('再次点击退出');
				setTimeout(() => {
					count = 0;
				}, 2000);
			}
		});
	});

	onUnmounted(() => {
		listener?.unregister();
	});
</script>
<style scoped lang = 'scss'>
	main {
		width: 100%;
		height: 100%;
		> div:nth-of-type(2) {
			width: 100%;
			height: calc(100% - 84px);
			position: relative;
			overflow: hidden;
			> div {
				position: absolute;
				top: 0;
				left: 0;
				width: 100%;
				height: 100%;
			}
		}
		> .var-tabs:last-of-type {
			position: fixed;
			bottom: 0;
			width: 100%;
		}
		.delete-cache-icon {
			margin-right: 12px;
		}
	}
	.move {
		&-enter-active,
		&-leave-active {
			transition: transform 0.2s ease;
		}

		&-enter-from {
			transform: translateX(100%);
		}

		&-leave-to {
			transform: translateX(-100%);
		}

		&-enter-to,
		&-leave-from {
			transform: translateX(0);
		}
	}
</style>
