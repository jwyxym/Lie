<template>
	<div class = 'anima'>
		<var-app-bar
			:title = 'ani?.name'
		>
			<template #left>
				<var-button
					round
					text
					@click = 'to_home'
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
				:src = 'ani.img'
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
						@click = 'to_video(i.url, route.query.url as string, ani.name)'
					>{{ i.name }}</var-button>
				</var-space>
			</div>
		</div>
	</div>
</template>
<script setup lang = 'ts'>
	import { watch, ref } from 'vue';
	import { useRoute } from 'vue-router';

	import { type AniInfo, get_ani } from '@/script/invoke';
	import { useTravel } from '@/script/travel';
	import themes from '@/script/themes';

	const { to_video, to_home } = useTravel();
	const route = useRoute();

	const ani = ref<AniInfo | undefined>(undefined);
	const select = ref<number>(0);

	watch(() => route.query.url, (url) => {
		get_ani(url as string).then(i => {
			ani.value = i;
		});
	}, { immediate : true });
</script>
<style scoped lang = 'scss'>
	.anima {
		width: 100%;
		height: 100%;
		:deep(.var-app-bar) {
			width: 100%;
			.var-app-bar__toolbar {
				width: 100%;
				.var-app-bar__left {
					width: calc(100% - 40px);
				}
			}
		}
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