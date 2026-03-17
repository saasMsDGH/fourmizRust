export const CommandType = {
  START: 0,
  PAUSE: 1,
  RESET: 2,
  SPAWN_FOOD: 3,
  SPAWN_PLANT: 4,
  SPAWN_ANIMAL: 5,
} as const;
export type CommandType = typeof CommandType[keyof typeof CommandType];

export const AntRole = {
  QUEEN: 0,
  WORKER: 1,
  SOLDIER: 2,
} as const;
export type AntRole = typeof AntRole[keyof typeof AntRole];

export const AntState = {
  IDLE: 0,
  EXPLORING: 1,
  RETURNING_WITH_FOOD: 2,
} as const;
export type AntState = typeof AntState[keyof typeof AntState];

export interface Ant {
  id: number;
  role: AntRole;
  x: number;
  y: number;
  state: AntState;
  angle: number;
  faction_id: number;
}

export const ResourceType = {
  PLANT: 0,
  ANIMAL: 1,
} as const;
export type ResourceType = typeof ResourceType[keyof typeof ResourceType];

export interface Resource {
  id: number;
  x: number;
  y: number;
  quantity: number;
  type: ResourceType;
}

export const RoomType = {
  ROYAL: 0,
  GRANARY: 1,
  NURSERY: 2,
} as const;
export type RoomType = typeof RoomType[keyof typeof RoomType];

export interface Room {
  id: number;
  room_type: RoomType;
  x: number;
  y: number;
  radius: number;
}

export interface Nest {
  faction_id: number;
  rooms: Room[];
}

export interface GameState {
  tick: number;
  ants: Ant[];
  resources: Resource[];
  nests: Nest[];
  encoded_pheromones: string; 
}
