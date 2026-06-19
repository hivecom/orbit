// Placeholder API that stays in sync with methods that Depot exposes

export const User = {
  uploadStart: () => {},
  upload: () => {},
  delete: () => {},
}

export const Internal = {
  keys: {
    get: () => {},
    post: () => {},
    delete: (keyId: string) => {
      void keyId
    },
  },
  quota: () => {},
  health: () => {},
}
