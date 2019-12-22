import { h } from '/static/js/html.js'
import { useTable } from '/static/js/depend/react-table.7.rc15.js'

export function Table({ columns, data }) {
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
                    h("th",column.getHeaderProps(), column.render('Header'))
                  ))
              )
          ))),
          h("tbody", getTableBodyProps(),
            rows.map((row, i) => {
                prepareRow(row);
                return (
                    h("tr", row.getRowProps(), row.cells.map(cell =>
                        h("td", cell.getCellProps(), cell.render('Cell'))))
                )}
            )
          ),
      ])
  )
}
