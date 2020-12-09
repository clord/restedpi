import React from 'react'
import 'twin.macro'

export function AccountDropdown() {
  return (
    <div tw="px-3 mt-6 relative inline-block text-left">
      <button
        type="button"
        tw="w-full bg-gray-100 rounded-md px-3.5 py-2 text-sm font-medium text-gray-700 hover:bg-gray-200 focus:outline-none focus:ring-2 focus:ring-offset-2 focus:ring-offset-gray-100 focus:ring-purple-500"
        id="options-menu"
        aria-haspopup="true"
        aria-expanded="true"
      >
        <span tw="flex w-full justify-between items-center">
          <span tw="flex min-w-0 items-center justify-between space-x-3">
            <span tw="flex-1 min-w-0">
              <span tw="text-gray-900 text-sm font-medium truncate">
                Christopher Lord
              </span>
            </span>
          </span>
        </span>
      </button>
    </div>
  )
}
