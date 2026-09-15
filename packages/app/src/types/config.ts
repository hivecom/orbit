export type KeyboardShortcuts = Record<
  any,
  {
    /**
     * Keys which trigger the shortcut.
     * For example "Ctrl+Shift+S"
     */
    keys: string
    /**
     * Called when shortcut triggers. It should be used to prevent default
     * browser behavior if needed.
     */
    handler?: (e: KeyboardEvent) => void
    /**
     * Summarizes what happens when shortcut is triggered.
     */
    title: string
    /**
     * Longer description for the shortcut.
     */
    description: string
  }
>

export type ShortcutCallback = () => void
