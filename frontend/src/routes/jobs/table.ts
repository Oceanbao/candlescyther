import { renderComponent, renderSnippet } from '$lib/components/ui/data-table';
import type { ColumnDef } from '@tanstack/table-core';
import { createRawSnippet } from 'svelte';
import DataTableActions from './data-table-action.svelte';
import DataTableSortButton from '$lib/components/data-table/data-table-sort-button.svelte';
import DataTableFilterSelect from '$lib/components/data-table/data-table-filter.svelte';
import { Checkbox } from '$lib/components/ui/checkbox/index.js';
import type { TJobs } from '$lib/server/client';

export const columns: ColumnDef<TJobs>[] = [
	{
		accessorKey: 'job_status',
		header: ({ column }) =>
			renderComponent(DataTableFilterSelect, {
				style: 'text-align: center; width: 100%; height: 100%;',
				filterValue: column.getFilterValue(),
				sortedUniqueValue: Array.from(column.getFacetedUniqueValues().keys()).sort(),
				onSelectChange: (e: Event) => {
					const target = e.target as HTMLSelectElement;
					column.setFilterValue(target.value);
				}
			}),
		cell: ({ row }) => {
			// const formatter = new Intl.NumberFormat('en-US', {
			// 	style: 'currency',
			// 	currency: 'USD'
			// });

			const cellSnippet = createRawSnippet<[{ job_status: string }]>((getData) => {
				const { job_status } = getData();
				// const formatted = formatter.format(amount);
				return {
					render: () => `<div class="text-right font-medium">${job_status}</div>`
				};
			});

			return renderSnippet(cellSnippet, {
				job_status: row.original.job_status
			});
		}
	},
	{
		accessorKey: 'job_type',
		header: () => {
			const headerSnippet = createRawSnippet(() => ({
				render: () => `<div class="text-center">Job Type</div>`
			}));
			return renderSnippet(headerSnippet);
		},
		cell: ({ row }) => {
			// const formatter = new Intl.NumberFormat('en-US', {
			// 	style: 'currency',
			// 	currency: 'USD'
			// });

			const cellSnippet = createRawSnippet<[{ job_type: string }]>((getData) => {
				const { job_type } = getData();
				// const formatted = formatter.format(amount);
				return {
					render: () => `<div class="text-right font-medium">${job_type}</div>`
				};
			});

			return renderSnippet(cellSnippet, {
				job_type: row.original.job_type
			});
		}
	},
	{
		accessorKey: 'payload',
		header: () => {
			const amountHeaderSnippet = createRawSnippet(() => ({
				render: () => `<div class="text-center">Payload</div>`
			}));
			return renderSnippet(amountHeaderSnippet);
		},
		cell: ({ row }) => {
			const payloadCellSnippet = createRawSnippet<[{ payload: string }]>((getPayload) => {
				const { payload } = getPayload();
				return {
					render: () => `<div class="text-right font-medium">${JSON.stringify(payload)}</div>`
				};
			});

			return renderSnippet(payloadCellSnippet, {
				payload: row.original.payload as string
			});
		},
		filterFn: (row, columnId, filterValue) => {
			let value = JSON.stringify(row.getValue(columnId) as Object);
			return value.includes(filterValue);
		}
	},
	{
		accessorKey: 'created_at',
		header: ({ column }) =>
			renderComponent(DataTableSortButton, {
				style: 'text-align: center; width: 100%; height: 100%;',
				onclick: column.getToggleSortingHandler()
			}),
		cell: ({ row }) => {
			const createdCellSnippet = createRawSnippet<[{ created_at: string }]>((getCreated) => {
				const { created_at } = getCreated();
				return {
					render: () => `<div class="text-center">${created_at}</div>`
				};
			});

			return renderSnippet(createdCellSnippet, {
				created_at: row.original.created_at
			});
		}
	},
	{
		accessorKey: 'error_message',
		header: () => {
			const amountHeaderSnippet = createRawSnippet(() => ({
				render: () => `<div class="text-center">Message</div>`
			}));
			return renderSnippet(amountHeaderSnippet);
		},
		cell: ({ row }) => {
			const payloadCellSnippet = createRawSnippet<[{ message: string }]>((getData) => {
				const { message } = getData();
				return {
					render: () => `<div class="text-right font-medium">${message}</div>`
				};
			});

			return renderSnippet(payloadCellSnippet, {
				message: row.original.error_message ?? ''
			});
		}
	},
	{
		id: 'actions',
		// You can access the row data using row.original in the cell function.
		// Use this to handle actions for your row eg. use the id to make a DELETE call to your API.
		cell: ({ row }) => {
			// You can pass whatever you need from `row.original` to the component
			return renderComponent(DataTableActions, { id: `${row.original.id}` });
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
