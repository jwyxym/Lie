<template>
	<var-card
		:title = 'item.name'
		:src = 'pic'
		:subtitle = 'item.count ?? item.desc'
		layout = 'row'
		@click = 'select'
	/>
</template>
<script setup lang = 'ts'>
	import { computed } from 'vue';
	import { type Item } from '@/script/invoke';
	import { useTravel } from '@/script/travel';

	const props : { item : Item; navigate : boolean; } = withDefaults(defineProps<{
		item : Item;
		navigate? : boolean;
	}>(), { navigate: true });
	const emit : (event : 'select') => void = defineEmits<{ select : []; }>();

	const pic = computed(() => props.item.img.replace('img2.xfmanga.top', 'img.lie.ygopro3.cn'))

	const { to_ani } = useTravel();
	const select = () : void => {
		if (props.navigate === false) { emit('select'); return; }
		void to_ani(props.item.url);
	};
</script>
<style scoped lang = 'scss'>
	.var-card {
		height: 100px;
		flex-shrink: 0;
	}
</style>
