<template>
	<TransitionGroup
		class = 'search'
		name = 'move'
		tag = 'div'
	>
		<div
			class = 'input'
			key = '0'
		>
			<var-input
				v-model = 'page.keywords'
				placeholder = '搜索'
				variant = 'outlined'
				size = 'small'
				clearable
				@keydown.enter = 'page.search($event)'
				@clear = 'page.clear()'
			/>
			<var-chip
				plain
				type = 'primary'
				@click = 'page.search()'
			>
				搜索
			</var-chip>
		</div>
		<var-list
			v-if = 'page.searched'
			class = 'results no-scrollbar'
			v-model:loading = 'page.loading'
			v-model:error = 'page.error'
			:finished = 'page.finished'
			:finished-text = "page.items.length ? '没有更多了' : '未找到相关番剧'"
			:immediate-check = 'false'
			@load = 'page.load()'
			key = '2'
		>
			<card
				v-for = 'item in page.items'
				:key = 'item.url'
				:item = 'item'
			/>
		</var-list>
	</TransitionGroup>
</template>
<script setup lang = 'ts'>
	import { onBeforeMount, onBeforeUnmount, reactive } from 'vue';
	import { get_search, type Items } from '@/script/invoke';
	import Card from '@/ui/card.vue';

	let request = 0;

	const page = reactive({
		keywords : '',
		query : '',
		ct : 1,
		loading : false,
		error : false,
		finished : false,
		searched : false,
		items : [] as Items,
		submitting : false,
		search (event ?: KeyboardEvent) {
			if (event?.isComposing)
				return;
			const keywords = this.keywords.trim();
			this.reset();
			this.query = keywords;
			emit('update:modelValue', keywords);
			if (!keywords)
				return;
			this.searched = true;
			void this.load();
		},
		reset () {
			request ++;
			this.ct = 1;
			this.items = [];
			this.loading = false;
			this.submitting = false;
			this.error = false;
			this.finished = false;
			this.searched = false;
		},
		clear () {
			this.reset();
			this.keywords = '';
			this.query = '';
			emit('update:modelValue', '');
		},
		async load () {
			if (!this.query || this.submitting || this.finished || this.error)
				return;
			const id = ++ request;
			this.loading = true;
			this.submitting = true;
			try {
				const result = await get_search(this.query, this.ct);
				if (id !== request)
					return;
				if (!result) {
					this.error = true;
					return;
				}
				this.items.push(...result.list);
				this.ct ++;
				this.finished = !result.hasMore;
			} finally {
				if (id === request) {
					this.loading = false;
					this.submitting = false;
				}
			}
		}
	});

	const props = defineProps<{
		modelValue : string;
	}>();

	const emit = defineEmits<{
		'update:modelValue' : [string];
	}>();

	onBeforeMount(() => {
		if (props.modelValue) {
			page.keywords = props.modelValue;
			page.search();
		}
	});

	onBeforeUnmount(() => {
		request ++;
	});
</script>
<style scoped lang = 'scss'>
	.search {
		width: 100%;
		height: 100%;
		position: relative;
		display: flex;
		flex-direction: column;
		align-items: center;
		gap: 10px;
		.input {
			height: 60px;
			width: 90%;
			display: flex;
			align-items: center;
			flex-wrap: wrap;
			gap: 10px;
			.var-chip {
				width: 60px;
				height: 30px;
				&:hover {
					cursor: pointer;
				}
			}
			.var-input {
				width: calc(100% - 80px);
			}
		}
		.results {
			position: absolute;
			top: 0;
			transform: translateX(var(--move-x, 0)) translateY(60px);
			height: calc(100% - 60px);
			width: 100%;
			overflow-y: auto;
			display: flex;
			flex-direction: column;
			gap: 5px;
			transition: all 0.2s ease;
		}
	}
	.move {
		&-enter-active,
		&-leave-active {
			transition: transform 0.2s ease;
		}

		&-enter-from {
			--move-x: 100%;
		}

		&-leave-to {
			--move-x: -100%;
		}

		&-enter-to,
		&-leave-from {
			--move-x: 0;
		}
	}
</style>
