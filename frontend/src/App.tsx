import { GlobalStyles } from 'twin.macro'
import React from 'react'
import { StaticSidebar } from './StaticSidebar'
import { MainColumn } from './MainColumn'
import { SignIn } from './SignIn'

interface SignedOut {
  type: 'signed-out'
}
interface SignedIn {
  type: 'signed-in'
}

type State = SignedOut | SignedIn

function Session() {
  const [state, setState] = React.useState<State>({ type: 'signed-out' })
  switch (state.type) {
    case 'signed-in': {
      return (
        <div tw="h-screen flex overflow-hidden bg-white">
          <StaticSidebar />
          <div tw="flex flex-col w-0 flex-1 overflow-hidden">
            <MainColumn />
          </div>
        </div>
      )
    }
    case 'signed-out':
      return <SignIn />
  }
}

export default function App() {
  return (
    <div>
      <GlobalStyles />
      <Session />
    </div>
  )
}
