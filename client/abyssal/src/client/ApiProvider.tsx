import { ReactNode, useCallback, useEffect, useMemo, useState } from "react";
import { useLocalStorage } from "@mantine/hooks";
import { GenericUser, getUserSelf, Uuid } from "./generated";
import { client } from "./generated/client.gen";
import { ApiContext } from "./util";

client.setConfig({
    baseURL: "/api",
    headers: (() => {
        const token = window.localStorage.getItem("abyssal.token");
        return token
            ? {
                  Authorization: `Token ${token}`,
              }
            : {};
    })(),
});

export function ApiProvider({
    children,
}: {
    children?: ReactNode | ReactNode[];
}) {
    const [storedToken, setStoredToken] = useLocalStorage<string | null>({
        key: "abyssal.token",
        defaultValue: null,
    });
    const [storedUser, setStoredUser] = useLocalStorage<GenericUser | null>({
        key: "abyssal.user",
        defaultValue: null,
    });
    const depStoredUser = useMemo(
        () => JSON.stringify(storedUser),
        [storedUser],
    );

    const [token, setToken] = useState(storedToken);
    const [user, setUser] = useState(storedUser);
    const depUser = JSON.stringify(user);

    useEffect(() => {
        setToken(storedToken);
        setUser(storedUser);
    }, [storedToken, depStoredUser]);

    const refresh = useCallback(() => {
        if (token === null) {
            setStoredToken(null);
            setStoredUser(null);
            client.setConfig({
                baseURL: "/api",
                headers: {},
            });
        } else {
            client.setConfig({
                baseURL: "/api",
                headers: { Authorization: `Token ${token}` },
            });
            getUserSelf().then((response) => {
                if (response.data) {
                    setStoredUser(response.data);
                } else {
                    setStoredToken(null);
                    setStoredUser(null);
                    client.setConfig({
                        baseURL: "/api",
                        headers: {},
                    });
                }
            });
        }
    }, [token, depUser, setStoredToken, setStoredUser]);

    useEffect(() => refresh(), []);

    const login = useCallback(
        (token: Uuid, user: GenericUser) => {
            setStoredToken(token);
            setStoredUser(user);
            client.setConfig({
                baseURL: "/api",
                headers: { Authorization: `Token ${token}` },
            });
        },
        [setStoredToken, setStoredUser],
    );

    const logout = useCallback(() => {
        setStoredToken(null);
        setStoredUser(null);
        client.setConfig({
            baseURL: "/api",
            headers: {},
        });
    }, [setStoredToken, setStoredUser]);

    return (
        <ApiContext.Provider
            value={
                token !== null && user !== null
                    ? { token: token as string, user, login, logout }
                    : { token: token as null, login, logout }
            }
        >
            {children}
        </ApiContext.Provider>
    );
}
