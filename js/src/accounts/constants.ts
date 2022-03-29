export enum AccountKey {
  Uninitialized = 0,
  Pass = 1,
  PassStore = 2,
  PassBook = 3,
}

export enum PassState {
  NotActivated = 0,
  Activated = 1,
  Deactivated = 2,
  Ended = 3,
}

export enum DurationType {
  Minutes = 0,
  Hours = 1,
  Days = 2,
  Unlimited = 3,
}
