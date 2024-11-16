import { atom } from 'jotai'

export const appState = atom<"default" | "hosting" | "connected">("default")