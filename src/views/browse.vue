<template>
	<div class = 'browse'>
		<var-cell :border = 'true' title = '状态'>
			<template #extra>
				<var-icon 
					:name = "browse.btn[0] ? 'chevron-down' : 'chevron-left'"
					:transition = '100'
					@click = '() => browse.btn[0] = !browse.btn[0]'
				/>
			</template>
		</var-cell>
		<var-collapse-transition :expand = 'browse.btn[0]'>
			<div>
				<var-button
					:type = "browse.status === 1 ? 'primary' : 'default'"
					@click = 'browse.select_status(1)'
				>
					连载中
				</var-button>
			</div>
			<div>
				<var-button
					:type = "browse.status === 2 ? 'primary' : 'default'"
					@click = 'browse.select_status(2)'
				>
					已完结
				</var-button>
			</div>
		</var-collapse-transition>
		<var-cell :border = 'true' title = '年份'>
			<template #extra>
				<var-icon 
					:name = "browse.btn[1] ? 'chevron-down' : 'chevron-left'"
					:transition = '100'
					@click = '() => browse.btn[1] = !browse.btn[1]'
				/>
			</template>
		</var-cell>
		<var-collapse-transition :expand = 'browse.btn[1]'>
			<div
				v-for = 'i in browse.years'
			>
				<var-button
					:type = "browse.year === i[1] ? 'primary' : 'default'"
					@click = 'browse.select_year(i[1])'
				>
					{{ i[0] }}
				</var-button>
			</div>
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
		btn : [false, false],
		years : [] as Array<[string, number]>,
		status : 1,
		year : 0,
		ct : 1,
		select_year (year : number) {
			if (this.year === year)
				return;
			this.year = year;
			this.ct = 1;
			this.finished = false;
			this.list.length = 0;
			//@ts-ignore
			list.value?.load?.();
		},
		select_status (status : number) {
			if (this.status === status)
				return;
			this.status = status;
			this.ct = 1;
			this.finished = false;
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
			get_browse(this.status, this.year, this.ct ++).then(i => {
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
		for (let i = year; i >= 2010; i --)
			browse.years.push([i.toString(), i]);
	});
</script>
<style scoped lang = 'scss'>
	.browse {
		color: var(--color-primary);
		> div {
			margin-top: 5px;
		}
		> div:first-of-type,
		> div:nth-of-type(3) {
			display: flex;
			gap: 20px;
			height: 30px;
		}
		.var-collapse-transition__content {
			display: flex;
			flex-wrap: wrap;
			gap: 5px;
			> * {
				width: 75px;
				height: 40px;
				display: flex;
				justify-content: center;
				align-items: center;
			}
		}
		.var-list {
			height: calc(100% - 100px);
			width: 100%;
			overflow-y: auto;
			display: flex;
			flex-direction: column;
			gap: 5px;
		}
	}
</style>