const routes = [
  {
    path: '/',
    component: () => import('@/pages/IndexPage.vue')
  },

  // Always leave this as last one
  {
    path: '/:catchAll(.*)*',
    redirect: '/'
  }
]

export default routes
