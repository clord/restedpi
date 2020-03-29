import { h } from "/js/html.js";
import { Fragment } from "/react/";
import { useTable, useSortBy } from "/js/depend/react-table.7.rc15.js";

const defaultPropGetter = () => ({});

export function Table({
  columns,
  data,
  getHeaderProps = defaultPropGetter,
  getColumnProps = defaultPropGetter,
  getRowProps = defaultPropGetter,
  getCellProps = defaultPropGetter
}) {
  const {
    getTableProps,
    getTableBodyProps,
    headerGroups,
    rows,
    prepareRow
  } = useTable(
    {
      columns,
      data
    },
    useSortBy
  );

  return h(
    "table",
    { ...getTableProps(), className: "border table-auto container" },
    [
      h(
        "thead",
        { key: "thead" },
        headerGroups.map(headerGroup =>
          h(
            "tr",
            headerGroup.getHeaderGroupProps(),
            headerGroup.headers.map(column =>
              h(
                "th",
                column.getHeaderProps([
                  {
                    className: `bg-gray-100 font-light text-gray-600 px-4 py-2 text-left ${
                      column.className == null ? "" : column.className
                    }`,
                    style: column.style
                  },
                  column.getSortByToggleProps(),
                  getColumnProps(column),
                  getHeaderProps(column)
                ]),
                [
                  h(Fragment, { key: "h" }, column.render("Header")),
                  h(
                    "span",
                    { key: "sort" },
                    column.isSorted ? (column.isSortedDesc ? " ⇟" : " ⇞") : "  "
                  )
                ]
              )
            )
          )
        )
      ),
      h(
        "tbody",
        { key: "tbody", ...getTableBodyProps() },
        rows.map((row, i) => {
          prepareRow(row);
          return h(
            "tr",
            row.getRowProps(),
            row.cells.map(cell =>
              h(
                "td",
                cell.getCellProps([
                  {
                    className: `${cell.column.className} border-t px-4 py-2`,
                    style: cell.column.style
                  },
                  getCellProps(cell)
                ]),
                cell.render("Cell")
              )
            )
          );
        })
      )
    ]
  );
}
