<template>
	<var-card
		:title = 'item.name'
		:src = 'item.img'
		:subtitle = 'item.count ?? item.desc'
		layout = 'row'
		@click = 'to_ani(item.url)'
	/>
</template>
<script setup lang = 'ts'>
	import { type Item } from '@/script/invoke';
	import { useTravel } from '@/script/travel';

	defineProps<{
		item : Item;
	}>();

	const { to_ani } = useTravel();
</script>
<style scoped lang = 'scss'>
	.var-card {
		height: 100px;
		flex-shrink: 0;
	}
</style>
