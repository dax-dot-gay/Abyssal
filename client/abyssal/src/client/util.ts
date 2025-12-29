import { createContext } from "react";
import { GenericUser, Uuid } from "./generated";

export type ApiContextType = (
    | {
          token: null;
      }
    | {
          token: Uuid;
          user: GenericUser;
      }
) & {
    login: (token: Uuid, user: GenericUser) => void;
    logout: () => void;
};

export const ApiContext = createContext<ApiContextType>({
    token: null,
    login: () => {},
    logout: () => {},
});
