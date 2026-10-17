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
	.var-app-bar {
		.var-app-bar__toolbar { padding: 0 6px; }
		.var-app-bar__left {
			position: static;
			flex: 1;
			min-width: 0;
			> .var-button { flex-shrink: 0; }
		}
		.var-app-bar__title {
			display: block;
			min-width: 0;
			line-height: var(--app-bar-height);
			text-align: left;
			white-space: nowrap;
			overflow: hidden;
			text-overflow: ellipsis;
		}
		.var-app-bar__right {
			position: static;
			flex-shrink: 0;
			margin-left: 12px;
		}
	}
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
