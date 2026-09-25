import { Suspense } from "react";

import { LoginScreen } from "@/widgets/login-screen";

export default function LoginPage() {
  return (
    <Suspense>
      <LoginScreen />
    </Suspense>
  );
}
