import { GlobalStyles } from 'twin.macro'
import React from 'react'
import { StaticSidebar } from './StaticSidebar'
import { MainColumn } from './MainColumn'

export default function App() {
  const authenticated = true
  if (authenticated) {
    return (
      <div>
        <GlobalStyles />
        <div tw="h-screen flex overflow-hidden bg-white">
          <StaticSidebar />
          <div tw="flex flex-col w-0 flex-1 overflow-hidden">
            <MainColumn />
          </div>
        </div>
      </div>
    )
  }
  return <div></div>
}
