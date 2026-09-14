<template>
	<main>
		<var-app-bar
			:title = "page.ct === 1 ? '全部番剧'
				: page.ct ? '搜素番剧'
					: '追番周表'"
		>
			<template #right>
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
			<schedule v-else :list = 'page.schedule.list' v-model = 'page.day'/>
		</TransitionGroup>
		<var-tabs
			v-model:active = 'page.ct'
		>
			<var-tab>追番周表</var-tab>
			<var-tab>全部番剧</var-tab>
			<var-tab>搜素番剧</var-tab>
		</var-tabs>
	</main>
</template>
<script setup lang = 'ts'>
	import { computed, onBeforeMount, onMounted, onUnmounted, reactive } from 'vue';
	import { useRoute } from 'vue-router';
	import { onBackButtonPress } from '@tauri-apps/api/app';
	import { exit } from '@tauri-apps/plugin-process';
import { Snackbar } from '@varlet/ui';

	import schedule from './schedule.vue';
	import browse from './browse.vue';
	import search from './search.vue';

	import { get_schedule, type Schedule } from '@/script/invoke';
	import themes from '@/script/themes';
	import { useTravel } from '@/script/travel';

	const { to_home } = useTravel();
	const route = useRoute();
	const page = reactive({
		ct : computed({
			get : () : number => parseInt(route.query.home as string | undefined ?? '0'),
			set : (i : number) => {
				to_home(i.toString(), (i > 0 ? 0 : page.day).toString(), page.search);
			}
		}),
		day : computed({
			get : () : number => parseInt(route.query.day as string | undefined ?? '0'),
			set : (i : number) => {
				to_home('0', i.toString(), page.search);
			}
		}),
		search : computed({
			get : () : string => route.query.search as string | undefined ?? '',
			set : (i : string) => {
				to_home('2', page.day.toString(), i);
			}
		}),
		schedule : {
			list : {} as Schedule
		}
	});

	onBeforeMount(async () => {
		page.schedule.list = await get_schedule();
	});

	let listener: Awaited<ReturnType<typeof onBackButtonPress>> | undefined;
	let count = 0;
	onMounted(async () => {
		listener = await onBackButtonPress(() => {
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
