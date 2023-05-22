export interface Config {
  valueCheck: (x: any) => any;
  unwrap: (x: any) => any;
  mangle?: boolean;
}

export function process(input: string, config: Config): string;
