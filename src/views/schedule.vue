<template>
	<div class = 'schedule'>
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
		<TransitionGroup
			name = 'move'
			tag = 'div'
			class = 'content'
			v-if = '!!list[0]'
		>
			<div
				v-for = 'v in [0, 1, 2, 3, 4, 5, 6]'
				class = 'no-scrollbar list'
				v-show = 'schedule.ct === v'
				:key = 'v'
			>
				<card
					v-for = 'i in list[v]'
					:item = 'i'
				/>
			</div>
		</TransitionGroup>
	</div>
</template>
<script setup lang = 'ts'>
	import { reactive, TransitionGroup } from 'vue';
	import { type Schedule } from '@/script/invoke';
	import card from '@/ui/card.vue';

	const schedule = reactive({
		ct : 0
	});

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
			width: 100%;
			height: calc(100% - 54px);
			position: relative;
			overflow: hidden;
			> .list {
				position: absolute;
				left: 0;
				top: 0;
				height: 100%;
				width: 100%;
				overflow-y: auto;
				display: flex;
				flex-direction: column;
				gap: 5px;
			}
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
