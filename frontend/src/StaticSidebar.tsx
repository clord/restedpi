import 'twin.macro'
import React from 'react'
import { Sidebar } from './Sidebar'

export function StaticSidebar() {
  return (
    <div tw="flex flex-shrink-0">
      <div tw="flex flex-col w-64 border-r border-gray-200 pt-5 pb-4 bg-gray-100">
        <div tw="flex items-center flex-shrink-0 px-6">RestedPi</div>
        <Sidebar />
      </div>
    </div>
  )
}
