import { createBrowserRouter } from "react-router";
import { LoginView } from "./views/login";
import { AppLayout } from "./views/layout/root";

export const AbyssalRouter = createBrowserRouter([
    {
        path: "/auth",
        element: <LoginView />,
    },
    {
        path: "/",
        element: <AppLayout />,
        children: [],
    },
]);
