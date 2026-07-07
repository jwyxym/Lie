<template>
	<div class = 'browse'>
		<div>
			年份
			<var-switch v-model = 'browse.btn'/>
		</div>
		<var-collapse-transition :expand = 'browse.btn'>
			<var-button
				v-for = 'i in browse.years'
				:type = "browse.year === i[1] ? 'primary' : 'default'"
				@click = 'browse.select(i[1])'
			>
				{{ i[0] }}
			</var-button>
		</var-collapse-transition>
		<var-list
			:finished = 'browse.finished'
			v-model:loading = 'browse.loading'
			@load = 'browse.load()'
			class = 'no-scrollbar'
			ref = 'list'
			@scroll = 'browse.load_on'
		>
			<card
				v-for = 'i in browse.list'
				:item = 'i'
			/>
		</var-list>
	</div>
</template>
<script setup lang = 'ts'>
	import { onBeforeMount, reactive, ref } from 'vue';
	import { get_browse, type Items } from '@/script/invoke';
	import card from '@/ui/card.vue';

	const list = ref(null);

	const browse = reactive({
		btn : false,
		years : [] as Array<[string, number]>,
		year : 0,
		ct : 0,
		select (year : number) {
			if (this.year === year)
				return;
			this.year = year;
			this.ct = 0;
			this.list.length = 0;
			//@ts-ignore
			list.value?.load?.();
		},
		loaded : false,
		loading : false,
		finished : false,
		load () {
			if (this.loaded)
				return;
			this.loaded = true;
			get_browse(this.year, this.ct ++).then(i => {
				i.length
					? this.list.push(...i)
					: this.finished = true;
				this.loading = false;
				this.loaded = false;
			});
		},
		load_on (event : Event) {
			const { scrollTop, scrollHeight, clientHeight } = event.target as HTMLElement;
			if (scrollHeight / browse.list.length < scrollHeight - scrollTop - clientHeight)
				return;
			browse.load();
		},
		list : [] as Items
	});

	onBeforeMount(() => {
		browse.years.push(['全部', 0]);
		const year = new Date().getFullYear();
		for (let i = year; i >= 2007; i --)
			browse.years.push([i.toString(), i]);
	});
</script>
<style scoped lang = 'scss'>
	.browse {
		> div {
			margin-top: 5px;
		}
		> div:first-of-type {
			display: flex;
			gap: 20px;
			height: 30px;
		}
		.var-collapse-transition {
			display: flex;
			flex-wrap: wrap;
			gap: 5px;
		}
		.var-list {
			height: calc(100% - 50px);
			width: 100%;
			overflow-y: auto;
			display: flex;
			flex-direction: column;
			gap: 5px;
		}
	}
</style>