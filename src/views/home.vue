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
			<search v-else-if = 'page.ct == 2'/>
			<schedule v-else :list = 'page.schedule.list'/>
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
	import { onBeforeMount, reactive } from 'vue';

	import schedule from './schedule.vue';
	import browse from './browse.vue';
	import search from './search.vue';

	import { get_schedule, type Schedule } from '@/script/invoke';
	import themes from '@/script/themes';

	const page = reactive({
		ct : 0,
		schedule : {
			list : {} as Schedule
		}
	});

	onBeforeMount(async () => {
		page.schedule.list = await get_schedule();
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
