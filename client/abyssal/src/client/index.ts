import * as SDK from "./generated";
import { ApiContext, ApiContextType } from "./util";
import { ApiProvider } from "./ApiProvider";
import { useContext } from "react";

export function useApi(): ApiContextType {
    return useContext(ApiContext);
}

export function useToken(): string | null {
    const api = useApi();
    return api.token;
}

export function useUser(): SDK.GenericUser | null {
    const api = useApi();
    return api.token ? api.user : null;
}

export type { ApiContextType };
export { SDK, ApiProvider };
