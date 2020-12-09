import React from 'react'
import 'twin.macro'
import { AccountDropdown } from './AccountDropdown'
import { SideNav } from './Nav'

export function Sidebar() {
  return (
    <div tw="h-0 flex-1 flex flex-col overflow-y-auto">
      <AccountDropdown />
      <SideNav />
    </div>
  )
}
