<template>
	<var-card
		:title = 'item.name'
		:src = 'item.img'
		:subtitle = 'item.count ?? item.desc'
		layout = 'row'
		@click = "to_ani(
			item.url,
			route.query.home as string | undefined ?? '0',
			route.query.day as string | undefined ?? '0',
			route.query.search as string | undefined ?? ''
		)"
	/>
</template>
<script setup lang = 'ts'>
	import { useRoute } from 'vue-router';
	import { type Item } from '@/script/invoke';
	import { useTravel } from '@/script/travel';

	const route = useRoute();
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
