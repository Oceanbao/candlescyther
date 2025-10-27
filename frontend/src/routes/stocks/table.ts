import { renderComponent, renderSnippet } from '$lib/components/ui/data-table';
import type { ColumnDef } from '@tanstack/table-core';
import { createRawSnippet } from 'svelte';
import DataTableActions from './data-table-action.svelte';
import DataTableSortButton from '$lib/components/data-table/data-table-sort-button.svelte';
import DataFilterSelect from './data-table-filter-select.svelte';
import { Checkbox } from '$lib/components/ui/checkbox/index.js';
import type { TStocks } from '$lib/server/client';

export const columns: ColumnDef<TStocks>[] = [
	{
		accessorKey: 'ticker',
		header: () => {
			const headerSnippet = createRawSnippet(() => ({
				render: () => `<div class="text-center">ticker</div>`
			}));
			return renderSnippet(headerSnippet);
		},
		cell: ({ row }) => {
			const cellSnippet = createRawSnippet<[{ ticker: string }]>((getData) => {
				const { ticker } = getData();
				return {
					render: () => `<div class="text-left ml-2 font-medium">${ticker}</div>`
				};
			});

			return renderSnippet(cellSnippet, {
				ticker: row.original.ticker
			});
		}
	},
	{
		accessorKey: 'market',
		filterFn: 'equalsString',
		header: ({ column }) =>
			renderComponent(DataFilterSelect, {
				style: 'text-align: center; width: 100%; height: 100%;',
				filterValue: column.getFilterValue(),
				sortedUniqueValue: Array.from(column.getFacetedUniqueValues().keys()).sort(),
				onSelectChange: (e: Event) => {
					const target = e.target as HTMLSelectElement;
					column.setFilterValue(target.value);
				}
			}),
		cell: ({ row }) => {
			const formatter = (market: number) => {
				if (market > 100 && market < 110) return 'US';
				if (market === 116) return 'H';
				return 'A';
			};
			const cellSnippet = createRawSnippet<[{ market: number }]>((getData) => {
				const { market } = getData();
				const formatted = formatter(market);
				return {
					render: () => `<div class="text-right font-medium">${formatted}</div>`
				};
			});

			return renderSnippet(cellSnippet, {
				market: row.original.market
			});
		}
	},
	{
		accessorKey: 'realname',
		header: () => {
			const headerSnippet = createRawSnippet(() => ({
				render: () => `<div class="text-center">name</div>`
			}));
			return renderSnippet(headerSnippet);
		},
		cell: ({ row }) => {
			const cellSnippet = createRawSnippet<[{ name: string }]>((getData) => {
				const { name } = getData();
				return {
					render: () => `<div class="text-right font-medium">${name}</div>`
				};
			});

			return renderSnippet(cellSnippet, {
				name: row.original.realname
			});
		}
	},
	{
		accessorKey: 'total_cap',
		header: ({ column }) =>
			renderComponent(DataTableSortButton, {
				name: 'cap',
				style: 'text-align: center; width: 100%; height: 100%;',
				onclick: column.getToggleSortingHandler()
			}),
		cell: ({ row }) => {
			const cellSnippet = createRawSnippet<[{ total_cap: number }]>((getData) => {
				let { total_cap } = getData();
				total_cap = total_cap / 1_000_000_000;
				return {
					render: () => `<div class="text-right">${total_cap.toFixed(4)}</div>`
				};
			});

			return renderSnippet(cellSnippet, {
				total_cap: row.original.total_cap ?? 0
			});
		}
	},
	{
		accessorKey: 'pe',
		header: ({ column }) =>
			renderComponent(DataTableSortButton, {
				name: 'PE',
				style: 'text-align: center; width: 100%; height: 100%;',
				onclick: column.getToggleSortingHandler()
			}),
		cell: ({ row }) => {
			const cellSnippet = createRawSnippet<[{ pe: number }]>((getData) => {
				let { pe } = getData();
				pe = pe / 100;
				return {
					render: () => `<div class="text-right">${pe}</div>`
				};
			});

			return renderSnippet(cellSnippet, {
				pe: row.original.pe ?? 0
			});
		}
	},
	{
		accessorKey: 'revenue',
		header: ({ column }) =>
			renderComponent(DataTableSortButton, {
				name: 'revenue',
				style: 'text-align: center; width: 100%; height: 100%;',
				onclick: column.getToggleSortingHandler()
			}),
		cell: ({ row }) => {
			const cellSnippet = createRawSnippet<[{ revenue: number }]>((getData) => {
				let { revenue } = getData();
				revenue = revenue / 1_000_000_000;
				return {
					render: () => `<div class="text-right font-medium">${revenue.toFixed(4)}</div>`
				};
			});

			return renderSnippet(cellSnippet, {
				revenue: row.original.revenue ?? 0
			});
		}
	},
	{
		accessorKey: 'net',
		header: ({ column }) =>
			renderComponent(DataTableSortButton, {
				name: 'net',
				style: 'text-align: center; width: 100%; height: 100%;',
				onclick: column.getToggleSortingHandler()
			}),
		cell: ({ row }) => {
			const cellSnippet = createRawSnippet<[{ net: number }]>((getData) => {
				let { net } = getData();
				net = net / 1_000_000_000;
				return {
					render: () => `<div class="text-right">${net.toFixed(4)}</div>`
				};
			});

			return renderSnippet(cellSnippet, {
				net: row.original.net ?? 0
			});
		}
	},
	{
		accessorKey: 'margin',
		header: ({ column }) =>
			renderComponent(DataTableSortButton, {
				name: 'margin',
				style: 'text-align: center; width: 100%; height: 100%;',
				onclick: column.getToggleSortingHandler()
			}),
		cell: ({ row }) => {
			const cellSnippet = createRawSnippet<[{ margin: number }]>((getData) => {
				let { margin } = getData();
				return {
					render: () => `<div class="text-right">${margin.toFixed(4)}</div>`
				};
			});

			return renderSnippet(cellSnippet, {
				margin: row.original.margin ?? 0
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
