import { h } from '/static/js/html.js'
import { useTable } from '/static/js/depend/react-table.7.rc15.js'

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
      h("table", getTableProps(), [
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
                                className: cell.column.className,
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
