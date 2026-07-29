export const WELCOME_COMPLETED_STORAGE_KEY = 'pilothub-welcome-completed-v1'

export const shouldShowWelcome = (completedValue: string | null) =>
  completedValue !== 'true'
