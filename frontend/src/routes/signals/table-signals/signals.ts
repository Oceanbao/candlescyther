import { renderComponent, renderSnippet } from '$lib/components/ui/data-table';
import type { ColumnDef } from '@tanstack/table-core';
import { createRawSnippet } from 'svelte';
import DataTableActions from './data-table-actions.svelte';
import DataTableSortButton from './data-table-sort-button.svelte';
import { Checkbox } from '$lib/components/ui/checkbox/index.js';
import type { TSignals } from '$lib/server/client';

const formatter = (num: number) => num.toFixed(4);

// This type is used to define the shape of data.
// Columns are where you define the core of what your table will look like.
// They define the data that will be displayed, how it will be formatted, sorted and filtered.
// Use Zod schema for typing.
export const columns: ColumnDef<TSignals>[] = [
	{
		accessorKey: 'ticker',
		header: () => {
			const headerSnippet = createRawSnippet(() => ({
				render: () => `<div class="text-center">Ticker</div>`
			}));
			return renderSnippet(headerSnippet);
		},
		cell: ({ row }) => {
			// const formatter = new Intl.NumberFormat('en-US', {
			// 	style: 'currency',
			// 	currency: 'USD'
			// });

			const cellSnippet = createRawSnippet<[{ ticker: string }]>((getData) => {
				const { ticker } = getData();
				// const formatted = formatter.format(amount);
				return {
					render: () => `<div class="text-center font-medium">${ticker}</div>`
				};
			});

			return renderSnippet(cellSnippet, {
				ticker: row.original.ticker
			});
		}
	},
	{
		accessorKey: 'boll_dist',
		header: ({ column }) =>
			renderComponent(DataTableSortButton, {
				style: 'text-align: center; width: 100%; height: 100%;',
				title: 'BOLL dist',
				onclick: column.getToggleSortingHandler()
			}),
		cell: ({ row }) => {
			const cellSnippet = createRawSnippet<[{ boll_dist: number }]>((getData) => {
				const { boll_dist } = getData();
				return {
					render: () => `<div class="text-center">${boll_dist.toFixed(4)}</div>`
				};
			});

			return renderSnippet(cellSnippet, {
				boll_dist: row.original.boll_dist
			});
		}
	},
	{
		accessorKey: 'kdj_k',
		header: () => {
			const headerSnippet = createRawSnippet(() => ({
				render: () => `<div class="text-center">K</div>`
			}));
			return renderSnippet(headerSnippet);
		},
		cell: ({ row }) => {
			const cellSnippet = createRawSnippet<[{ kdj_k: number }]>((getData) => {
				const { kdj_k } = getData();
				const formatted = formatter(kdj_k);
				return {
					render: () => `<div class="text-center font-medium">${formatted}</div>`
				};
			});

			return renderSnippet(cellSnippet, {
				kdj_k: row.original.kdj_k
			});
		}
	},
	{
		accessorKey: 'kdj_d',
		header: ({ column }) =>
			renderComponent(DataTableSortButton, {
				style: 'text-align: center; width: 100%; height: 100%;',
				title: 'D',
				onclick: column.getToggleSortingHandler()
			}),
		cell: ({ row }) => {
			const cellSnippet = createRawSnippet<[{ kdj_d: number }]>((getData) => {
				const { kdj_d } = getData();
				const formatted = formatter(kdj_d);
				return {
					render: () => `<div class="text-center">${formatted}</div>`
				};
			});

			return renderSnippet(cellSnippet, {
				kdj_d: row.original.kdj_d
			});
		}
	},
	{
		id: 'actions',
		// You can access the row data using row.original in the cell function.
		// Use this to handle actions for your row eg. use the id to make a DELETE call to your API.
		cell: ({ row }) => {
			// You can pass whatever you need from `row.original` to the component
			return renderComponent(DataTableActions, { id: `${row.original.ticker}` });
		}
	},
	{
		id: 'select',
		header: ({ table }) =>
			renderComponent(Checkbox, {
				checked: table.getIsAllPageRowsSelected(),
				indeterminate: table.getIsSomePageRowsSelected() && !table.getIsAllPageRowsSelected(),
				onCheckedChange: (value) => table.toggleAllPageRowsSelected(!!value),
				'aria-label': 'Select all'
			}),
		cell: ({ row }) =>
			renderComponent(Checkbox, {
				checked: row.getIsSelected(),
				onCheckedChange: (value) => row.toggleSelected(!!value),
				'aria-label': 'Select row'
			}),
		enableSorting: false,
		enableHiding: false
	}
];
