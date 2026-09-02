import { createFileRoute } from '@tanstack/react-router'

import { App } from '../App'

const IndexComponent = App

export const Route = createFileRoute('/')({
  component: IndexComponent,
})
