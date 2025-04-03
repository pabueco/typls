export type Expansion = {
  id?: string;
  abbr: string;
  text: string;
  group?: string;
};

export type Group = {
  id: string;
  name: string;
  apps: string[];
};

export type Settings = {
  trigger: {
    string: string;
  };
  confirm: {
    chars: string[];
    keyEnter: boolean;
    keyRightArrow: boolean;
    append: boolean;
    auto: boolean;
  };
  variables: {
    separator: string;
  };
  expansions: Expansion[];
  groups: Group[];
};
