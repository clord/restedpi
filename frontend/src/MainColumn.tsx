import React from 'react'
import 'twin.macro'

export function MainColumn() {
  return (
    <main tw="flex-1 relative z-0 overflow-y-auto focus:outline-none" tabIndex={0}>
      <div tw="border-b border-gray-200 px-4 py-4 sm:flex sm:items-center sm:justify-between sm:px-6 lg:px-8">
        <div tw="flex-1 min-w-0">
          <h1 tw="text-lg font-medium leading-6 text-gray-900 sm:truncate">Home</h1>
        </div>
        <div tw="mt-4 flex sm:mt-0 sm:ml-4">
          <button
            type="button"
            tw="order-none inline-flex items-center px-4 py-2 border border-transparent shadow-sm text-sm font-medium rounded-md text-white bg-purple-600 hover:bg-purple-700 focus:outline-none focus:ring-2 focus:ring-offset-2 focus:ring-purple-500 sm:order-1 sm:ml-3"
          >
            New Device
          </button>
        </div>
      </div>
      <div tw="px-4 mt-6 sm:px-6 lg:px-8">
        <h2 tw="text-gray-500 text-xs font-medium uppercase tracking-wide">
          Pinned Projects
        </h2>
        <ul tw="grid grid-cols-1 gap-4 sm:gap-6 sm:grid-cols-2 xl:grid-cols-4 mt-3">
          <li tw="relative col-span-1 flex shadow-sm rounded-md">
            <div tw="flex-shrink-0 flex items-center justify-center w-16 bg-pink-600 text-white text-sm font-medium rounded-l-md">
              GA
            </div>
            <div tw="flex-1 flex items-center justify-between border-t border-r border-b border-gray-200 bg-white rounded-r-md truncate">
              <div tw="flex-1 px-4 py-2 text-sm truncate">
                <a href="#" tw="text-gray-900 font-medium hover:text-gray-600">
                  GraphQL API
                </a>
                <p tw="text-gray-500">12 Members</p>
              </div>
              <div tw="flex-shrink-0 pr-2">
                <button
                  id="pinned-project-options-menu-0"
                  aria-haspopup="true"
                  tw="w-8 h-8 bg-white inline-flex items-center justify-center text-gray-400 rounded-full hover:text-gray-500 focus:outline-none focus:ring-2 focus:ring-offset-2 focus:ring-purple-500"
                >
                  <span tw="sr-only">Open options</span>
                  <svg
                    tw="w-5 h-5"
                    xmlns="http://www.w3.org/2000/svg"
                    viewBox="0 0 20 20"
                    fill="currentColor"
                    aria-hidden="true"
                  >
                    <path d="M10 6a2 2 0 110-4 2 2 0 010 4zM10 12a2 2 0 110-4 2 2 0 010 4zM10 18a2 2 0 110-4 2 2 0 010 4z" />
                  </svg>
                </button>
              </div>
            </div>
          </li>
        </ul>
      </div>

      <div tw="mt-10 sm:hidden">
        <div tw="px-4 sm:px-6">
          <h2 tw="text-gray-500 text-xs font-medium uppercase tracking-wide">Projects</h2>
        </div>
        <ul tw="mt-3 border-t border-gray-200 divide-y divide-gray-100">
          <li>
            <a
              href="#"
              tw="flex items-center justify-between px-4 py-4 hover:bg-gray-50 sm:px-6"
            >
              <span tw="flex items-center truncate space-x-3">
                <span
                  tw="w-2.5 h-2.5 flex-shrink-0 rounded-full bg-pink-600"
                  aria-hidden="true"
                ></span>
                <span tw="font-medium truncate text-sm leading-6">
                  GraphQL API
                  <span tw="truncate font-normal text-gray-500">in Engineering</span>
                </span>
              </span>
              <svg
                tw="ml-4 h-5 w-5 text-gray-400 "
                xmlns="http://www.w3.org/2000/svg"
                viewBox="0 0 20 20"
                fill="currentColor"
                aria-hidden="true"
              >
                <path
                  fillRule="evenodd"
                  d="M7.293 14.707a1 1 0 010-1.414L10.586 10 7.293 6.707a1 1 0 011.414-1.414l4 4a1 1 0 010 1.414l-4 4a1 1 0 01-1.414 0z"
                  clipRule="evenodd"
                />
              </svg>
            </a>
          </li>
        </ul>
      </div>

      <div tw="hidden mt-8 sm:block">
        <div tw="align-middle inline-block min-w-full border-b border-gray-200">
          <table tw="min-w-full">
            <thead>
              <tr tw="border-t border-gray-200">
                <th tw="px-6 py-3 border-b border-gray-200 bg-gray-50 text-left text-xs font-medium text-gray-500 uppercase tracking-wider">
                  <span tw="lg:pl-2">Project</span>
                </th>
                <th tw="px-6 py-3 border-b border-gray-200 bg-gray-50 text-left text-xs font-medium text-gray-500 uppercase tracking-wider">
                  Members
                </th>
                <th tw="hidden md:table-cell px-6 py-3 border-b border-gray-200 bg-gray-50 text-right text-xs font-medium text-gray-500 uppercase tracking-wider">
                  Last updated
                </th>
                <th tw="pr-6 py-3 border-b border-gray-200 bg-gray-50 text-right text-xs font-medium text-gray-500 uppercase tracking-wider"></th>
              </tr>
            </thead>
            <tbody tw="bg-white divide-y divide-gray-100">
              <tr>
                <td tw="px-6 py-3 max-w-0 w-full whitespace-nowrap text-sm font-medium text-gray-900">
                  <div tw="flex items-center space-x-3 lg:pl-2">
                    <div
                      tw="flex-shrink-0 w-2.5 h-2.5 rounded-full bg-pink-600"
                      aria-hidden="true"
                    ></div>
                    <a href="#" tw="truncate hover:text-gray-600">
                      <span>
                        GraphQL API
                        <span tw="text-gray-500 font-normal">in Engineering</span>
                      </span>
                    </a>
                  </div>
                </td>
                <td tw="px-6 py-3 text-sm text-gray-500 font-medium">
                  <div tw="flex items-center space-x-2">
                    <div tw="flex flex-shrink-0 -space-x-1">
                      <img
                        tw="max-w-none h-6 w-6 rounded-full ring-2 ring-white"
                        src="https://images.unsplash.com/photo-1502685104226-ee32379fefbe?ixlib=rb-1.2.1&ixid=eyJhcHBfaWQiOjEyMDd9&auto=format&fit=facearea&facepad=2&w=256&h=256&q=80"
                        alt=""
                      />

                      <img
                        tw="max-w-none h-6 w-6 rounded-full ring-2 ring-white"
                        src="https://images.unsplash.com/photo-1491528323818-fdd1faba62cc?ixlib=rb-1.2.1&ixid=eyJhcHBfaWQiOjEyMDd9&auto=format&fit=facearea&facepad=2&w=256&h=256&q=80"
                        alt=""
                      />

                      <img
                        tw="max-w-none h-6 w-6 rounded-full ring-2 ring-white"
                        src="https://images.unsplash.com/photo-1550525811-e5869dd03032?ixlib=rb-1.2.1&ixid=eyJhcHBfaWQiOjEyMDd9&auto=format&fit=facearea&facepad=2&w=256&h=256&q=80"
                        alt=""
                      />

                      <img
                        tw="max-w-none h-6 w-6 rounded-full ring-2 ring-white"
                        src="https://images.unsplash.com/photo-1500648767791-00dcc994a43e?ixlib=rb-1.2.1&ixid=eyJhcHBfaWQiOjEyMDd9&auto=format&fit=facearea&facepad=2&w=256&h=256&q=80"
                        alt=""
                      />
                    </div>
                  </div>
                </td>
                <td tw="hidden md:table-cell px-6 py-3 whitespace-nowrap text-sm text-gray-500 text-right">
                  March 17, 2020
                </td>
                <td tw="pr-6">
                  <div tw="relative flex justify-end items-center">
                    <button
                      id="project-options-menu-0"
                      aria-haspopup="true"
                      type="button"
                      tw="w-8 h-8 bg-white inline-flex items-center justify-center text-gray-400 rounded-full hover:text-gray-500 focus:outline-none focus:ring-2 focus:ring-offset-2 focus:ring-purple-500"
                    >
                      <span tw="sr-only">Open options</span>
                      <svg
                        tw="w-5 h-5"
                        xmlns="http://www.w3.org/2000/svg"
                        viewBox="0 0 20 20"
                        fill="currentColor"
                        aria-hidden="true"
                      >
                        <path d="M10 6a2 2 0 110-4 2 2 0 010 4zM10 12a2 2 0 110-4 2 2 0 010 4zM10 18a2 2 0 110-4 2 2 0 010 4z" />
                      </svg>
                    </button>
                  </div>
                </td>
              </tr>
            </tbody>
          </table>
        </div>
      </div>
    </main>
  )
}
