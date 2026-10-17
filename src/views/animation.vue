<template>
	<div class = 'anima'>
		<var-app-bar
			:title = 'ani?.name'
			title-position = 'left'
		>
			<template #left>
				<var-button
					round
					text
					@click = 'to_home(
						route.query.home as string,
						route.query.day as string,
						route.query.search as string
					)'
				>
					<var-icon
						name = 'chevron-left'
						:size = '24'
					/>
				</var-button>
			</template>
			<template #right>
				<var-icon 
					namespace = 'i'
					:name = 'themes.name'
					:transition = '300'
					@click = 'themes.change()'
				/>
			</template>
		</var-app-bar>
		<var-skeleton :loading = '!ani'/>
		<div
			class = 'content no-scrollbar'
			v-if = 'ani'
		>
			<var-card
				:title = 'ani.name'
				:subtitle = 'ani.date'
				:description = 'ani.desc'
				:src = 'pic'
			>
			</var-card>
			<div
				class = 'list'
			>
				<var-tabs
					v-model:active = 'select'
					color = 'transparent'
				>
					<var-tab v-for = '(_, v) in ani.links'>
						播放来源{{ v + 1 }}
					</var-tab>
				</var-tabs>
				<var-space>
					<var-button
						v-for = 'i in ani.links[select]'
						type = 'primary'
						@click = 'to_video(
							i.url,
							route.query.url as string,
							ani.name, select,
							route.query.home as string,
							route.query.day as string,
							route.query.search as string,
							ani.img
						)'
					>{{ i.name }}</var-button>
				</var-space>
			</div>
		</div>
	</div>
</template>
<script setup lang = 'ts'>
	import { watch, ref, onUnmounted, onMounted, computed } from 'vue';
	import { useRoute } from 'vue-router';
	import { onBackButtonPress } from '@tauri-apps/api/app';

	import { type AniInfo, get_ani } from '@/script/invoke';
	import { useTravel } from '@/script/travel';
	import themes from '@/script/themes';

	const { to_video, to_home } = useTravel();
	const route = useRoute();

	const ani = ref<AniInfo | undefined>(undefined);
	const select = ref<number>(0);
	const pic = computed(() => ani.value?.img.replace('img2.xfmanga.top', 'img.lie.ygopro3.cn') ?? '');

	watch(() => route.query.url, (url) => {
		get_ani(url as string).then(i => {
			ani.value = i;
		});
	}, { immediate : true });

	let listener: Awaited<ReturnType<typeof onBackButtonPress>> | undefined;
	onMounted(async () => {
		listener = await onBackButtonPress((i) => {
			if (i.canGoBack)
				to_home(
					route.query.home as string,
					route.query.day as string,
					route.query.search as string
				);
		});
	});

	onUnmounted(() => {
		listener?.unregister();
	});
</script>
<style scoped lang = 'scss'>
	.anima {
		width: 100%;
		height: 100%;
		.content {
			width: 100%;
			height: calc(100% - 54px);
			overflow-y: auto;
			.var-card {
				width: 100%;
			}
			.list {
				width: 100%;
				display: flex;
				flex-direction: column;
				gap: 5px;
				.var-space {
					width: 90%;
					align-content: flex-start;
				}
			}
		}
	}
</style>
