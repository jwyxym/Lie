<template>
	<div
		class = 'schedule'
		@touchstart.passive = 'touch_start'
		@touchend.passive = 'touch_end'
	>
		<var-tabs
			v-model:active = 'schedule.ct'
		>
			<var-tab v-for = "i in [
				'一',
				'二',
				'三',
				'四',
				'五',
				'六',
				'日'
			]">{{ i }}</var-tab>
		</var-tabs>
		<var-skeleton :loading = '!list[0]'/>
		<div
			v-if = '!!list[0]'
			class = 'content'
			:style = "{ '--left' : schedule.ct}"
		>
			<div
				v-for = 'v in [0, 1, 2, 3, 4, 5, 6]'
				class = 'no-scrollbar list'
				:key = 'v'
			>
				<card
					v-for = 'i in list[v]'
					:item = 'i'
				/>
			</div>
		</div>
	</div>
</template>
<script setup lang = 'ts'>
	import { reactive } from 'vue';
	import { type Schedule } from '@/script/invoke';
	import card from '@/ui/card.vue';

	const schedule = reactive({
		ct : 0
	});

	const touch = reactive({
		x : 0,
		y : 0
	});

	function touch_start(e : TouchEvent) {
		const point = e.changedTouches[0];
		touch.x = point.clientX;
		touch.y = point.clientY;
	}

	function touch_end(e : TouchEvent) {
		const point = e.changedTouches[0];
		const x = point.clientX - touch.x;
		const y = point.clientY - touch.y;

		if (Math.abs(x) < 50 || Math.abs(x) < Math.abs(y))
			return;

		if (x < 0)
			schedule.ct = Math.min(schedule.ct + 1, 6);
		else
			schedule.ct = Math.max(schedule.ct - 1, 0);
	}

	defineProps<{
		list : Schedule
	}>();
</script>
<style scoped lang = 'scss'>
	.schedule {
		> .var-skeleton {
			width: 100%;
		}
		> .content {
			width: calc(100% * 7);
			height: calc(100% - 54px);
			overflow: hidden;
			transform: translateX(calc(var(--width) * -1 * var(--left)));
			transition: all 0.3s ease;
			display: flex;
			> .list {
				height: 100%;
				width: var(--width);
				overflow-y: auto;
				display: flex;
				flex-direction: column;
				gap: 5px;
			}
		}
	}
</style>
