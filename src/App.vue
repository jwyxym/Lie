<template>
	<router-view/>
</template>
<script setup lang = 'ts'>
	import { onBeforeMount } from 'vue';
	import { Themes, StyleProvider } from '@varlet/ui';

	onBeforeMount(() => {
		StyleProvider(Themes.md3Light);
	});
</script>
<style lang = 'scss'>
	html,
	body {
		margin: 0;
		width: var(--width);
		height: var(--height);
	}
	body {
		background-color: var(--color-body);
		#app {
			margin: 0;
			width: var(--width);
			height: calc(var(--height) - 74px);
			transform: translateY(30px);
		}
	}
	.no-scrollbar {
		&::-webkit-scrollbar {
			display: none;
		}
	}
</style>