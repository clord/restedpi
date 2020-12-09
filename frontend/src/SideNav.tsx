import React from 'react'
import 'twin.macro'

export function SideNav() {
  return (
    <nav tw="px-3 mt-6">
      <div tw="space-y-1">
        <a
          href="#"
          tw="bg-gray-200 text-gray-900 flex items-center px-2 py-2 text-sm font-medium rounded-md"
        >
          <svg
            tw="text-gray-500 mr-3 h-6 w-6"
            xmlns="http://www.w3.org/2000/svg"
            fill="none"
            viewBox="0 0 24 24"
            stroke="currentColor"
            aria-hidden="true"
          >
            <path
              strokeLinecap="round"
              strokeLinejoin="round"
              strokeWidth="2"
              d="M3 12l2-2m0 0l7-7 7 7M5 10v10a1 1 0 001 1h3m10-11l2 2m-2-2v10a1 1 0 01-1 1h-3m-6 0a1 1 0 001-1v-4a1 1 0 011-1h2a1 1 0 011 1v4a1 1 0 001 1m-6 0h6"
            />
          </svg>
          Home
        </a>

        <a
          href="#"
          tw="text-gray-700 hover:text-gray-900 hover:bg-gray-50 flex items-center px-2 py-2 text-sm font-medium rounded-md"
        >
          <svg
            tw="text-gray-400 mr-3 h-6 w-6"
            xmlns="http://www.w3.org/2000/svg"
            fill="none"
            viewBox="0 0 24 24"
            stroke="currentColor"
            aria-hidden="true"
          >
            <path
              strokeLinecap="round"
              strokeLinejoin="round"
              strokeWidth="2"
              d="M4 6h16M4 10h16M4 14h16M4 18h16"
            />
          </svg>
          Devices
        </a>

        <a
          href="#"
          tw="text-gray-700 hover:text-gray-900 hover:bg-gray-50 flex items-center px-2 py-2 text-sm font-medium rounded-md"
        >
          <svg
            tw="text-gray-400 mr-3 h-6 w-6"
            xmlns="http://www.w3.org/2000/svg"
            fill="none"
            viewBox="0 0 24 24"
            stroke="currentColor"
            aria-hidden="true"
          >
            <path
              strokeLinecap="round"
              strokeLinejoin="round"
              strokeWidth="2"
              d="M12 8v4l3 3m6-3a9 9 0 11-18 0 9 9 0 0118 0z"
            />
          </svg>
          Inputs
        </a>
        <a
          href="#"
          tw="text-gray-700 hover:text-gray-900 hover:bg-gray-50 flex items-center px-2 py-2 text-sm font-medium rounded-md"
        >
          <svg
            tw="text-gray-400 mr-3 h-6 w-6"
            xmlns="http://www.w3.org/2000/svg"
            fill="none"
            viewBox="0 0 24 24"
            stroke="currentColor"
            aria-hidden="true"
          >
            <path
              strokeLinecap="round"
              strokeLinejoin="round"
              strokeWidth="2"
              d="M12 8v4l3 3m6-3a9 9 0 11-18 0 9 9 0 0118 0z"
            />
          </svg>
          Outputs
        </a>
      </div>
    </nav>
  )
}
