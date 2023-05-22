export interface Accessor {
  kind: "function" | "property" | "method";
  value: string;
}

export interface Config {
  valueCheck: Accessor;
  unwrap: Accessor;
  mangle?: boolean;
}

export function process(input: string, config: Config): string;
