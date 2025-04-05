export type Expansion = {
  id?: string;
  abbr: string;
  text: string;
  group?: string | null;
};

export type App = {
  path: string;
  os: string;
};

export type Group = {
  id: string;
  name: string;
  apps: App[];
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
  activeGroup: Group["id"];
};
