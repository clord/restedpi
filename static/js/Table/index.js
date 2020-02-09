import { h } from '/js/html.js'
import { useTable } from '/js/depend/react-table.7.rc15.js'

const defaultPropGetter = () => ({})

export function Table({ columns, data,
  getHeaderProps = defaultPropGetter,
  getColumnProps = defaultPropGetter,
  getRowProps = defaultPropGetter,
  getCellProps = defaultPropGetter,
}) {
  const {
    getTableProps,
    getTableBodyProps,
    headerGroups,
    rows,
    prepareRow,
  } = useTable({
    columns,
    data,
  })

  return (
      h("table", {...getTableProps(), class: "border table-auto container"}, [
          h("thead", {}, headerGroups.map(headerGroup => (
              h("tr", headerGroup.getHeaderGroupProps(),
                  headerGroup.headers.map(column => (
                      h("th", column.getHeaderProps([{
                            className: column.className,
                            style: column.style,
                        },
                        getColumnProps(column),
                        getHeaderProps(column),
                      ]), column.render('Header'))
                  ))
              )
          ))),
          h("tbody", getTableBodyProps(),
            rows.map((row, i) => {
                prepareRow(row);
                return (
                    h("tr", row.getRowProps(), row.cells.map(cell =>
                        h("td", cell.getCellProps([{
                                className: `${cell.column.className} border-t px-4 py-2`,
                                style: cell.column.style,
                            },
                            getCellProps(cell),
                        ]), cell.render('Cell'))))
                )}
            )
          ),
      ])
  )
}
