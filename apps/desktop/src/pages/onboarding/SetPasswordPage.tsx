import { useEffect, useState } from "react";
import { useNavigate } from "react-router-dom";
import { PasswordRequirements } from "@/components/PasswordRequirements";
import { Button } from "@/components/ui/button";
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from "@/components/ui/card";
import { Input } from "@/components/ui/input";
import { Label } from "@/components/ui/label";
import { Alert } from "@/components/ui/misc";
import { Checkbox } from "@/components/ui/checkbox";
import { useWallet } from "@/context/WalletContext";
import { isPasswordStrong, passwordStrengthError } from "@/lib/password";
import { ApiError, walletApi } from "@/lib/tauri";

type Mode = "create" | "import" | "import-key";

export function SetPasswordPage() {
  const navigate = useNavigate();
  const { unlock, refresh } = useWallet();
  const [password, setPassword] = useState("");
  const [confirm, setConfirm] = useState("");
  const [accountName, setAccountName] = useState("Account 1");
  const [error, setError] = useState<string | null>(null);
  const [loading, setLoading] = useState(false);
  const [mnemonic, setMnemonic] = useState("");
  const [secret, setSecret] = useState("");
  const [mode, setMode] = useState<Mode>("create");
  const [ready, setReady] = useState(false);
  const [enableDeviceProtection, setEnableDeviceProtection] = useState(false);

  useEffect(() => {
    let cancelled = false;
    void (async () => {
      try {
        const draft = await walletApi.getOnboardingDraft();
        if (cancelled) return;
        const nextMode = draft?.mode === "import-key"
          ? "import-key"
          : draft?.mode === "import"
            ? "import"
            : "create";
        if (nextMode === "import-key") {
          if (!draft?.secret) {
            navigate("/onboarding", { replace: true });
            return;
          }
          setSecret(draft.secret);
        } else if (!draft?.mnemonic) {
          navigate("/onboarding", { replace: true });
          return;
        } else {
          setMnemonic(draft.mnemonic);
        }
        setMode(nextMode);
        if (draft?.account_name) setAccountName(draft.account_name);
        setReady(true);
      } catch {
        if (!cancelled) navigate("/onboarding", { replace: true });
      }
    })();
    return () => {
      cancelled = true;
    };
  }, [navigate]);

  const passwordsMatch = confirm.length > 0 && password === confirm;
  const canSubmit = isPasswordStrong(password) && passwordsMatch && !loading;

  const handleSubmit = async (event: React.FormEvent) => {
    event.preventDefault();
    const strengthError = passwordStrengthError(password);
    if (strengthError) {
      setError(strengthError);
      return;
    }
    if (password !== confirm) {
      setError("Passwords do not match");
      return;
    }

    setLoading(true);
    setError(null);
    const name = accountName.trim() || "Account 1";
    try {
      if (mode === "import-key") {
        await walletApi.importPrivateKey(secret, password, name);
      } else if (mode === "import") {
        await walletApi.importWallet(mnemonic, password, name);
      } else {
        await walletApi.createWallet(mnemonic, password, name);
      }
      await walletApi.clearOnboardingDraft();

      if (enableDeviceProtection) {
        try {
          await walletApi.enableDeviceProtection(password);
        } catch (protErr) {
          const apiError = protErr as ApiError;
          await unlock(password);
          navigate("/settings/security", {
            replace: true,
            state: {
              notice:
                apiError.message ??
                "Wallet is ready, but Enhanced device protection could not be enabled. Turn it on below.",
            },
          });
          return;
        }
      }

      await unlock(password);
      navigate("/onboarding/ready");
    } catch (err) {
      const apiError = err as ApiError;
      setError(apiError.message ?? "Failed to create wallet");
      try {
        const snap = await walletApi.getWalletSnapshot();
        if (snap.exists) await refresh();
      } catch {
        // ignore snapshot errors
      }
    } finally {
      setLoading(false);
    }
  };

  if (!ready) {
    return null;
  }

  const submitLabel =
    mode === "import-key" ? "Import key" : mode === "import" ? "Import wallet" : "Create wallet";

  return (
    <div className="flex min-h-screen items-center justify-center bg-background p-6">
      <Card className="w-full max-w-md">
        <CardHeader>
          <CardTitle>Set password</CardTitle>
          <CardDescription>This password encrypts your wallet file on this device.</CardDescription>
        </CardHeader>
        <CardContent>
          <form className="space-y-4" onSubmit={handleSubmit}>
            <div className="space-y-2">
              <Label htmlFor="account-name">Account name</Label>
              <Input
                id="account-name"
                value={accountName}
                onChange={(e) => setAccountName(e.target.value)}
                placeholder="Account 1"
                autoComplete="off"
              />
            </div>
            <div className="space-y-2">
              <Label htmlFor="password">Password</Label>
              <Input
                id="password"
                type="password"
                autoComplete="new-password"
                value={password}
                onChange={(e) => setPassword(e.target.value)}
              />
              <PasswordRequirements password={password} className="pt-1" />
            </div>
            <div className="space-y-2">
              <Label htmlFor="confirm">Confirm password</Label>
              <Input
                id="confirm"
                type="password"
                autoComplete="new-password"
                value={confirm}
                onChange={(e) => setConfirm(e.target.value)}
              />
              {confirm.length > 0 && (
                <p
                  className={
                    passwordsMatch
                      ? "text-xs text-emerald-600 dark:text-emerald-400"
                      : "text-xs text-destructive"
                  }
                >
                  {passwordsMatch ? "Passwords match" : "Passwords do not match"}
                </p>
              )}
            </div>
            <label className="flex cursor-pointer items-start gap-2.5 text-sm">
              <Checkbox
                checked={enableDeviceProtection}
                onCheckedChange={setEnableDeviceProtection}
                aria-label="Enable Enhanced device protection"
              />
              <span>
                Enable Enhanced device protection
                <span className="mt-1 block text-xs text-muted-foreground">
                  Binds decryption to this device. Confirm you have a backup first.
                </span>
              </span>
            </label>
            {error && <Alert className="border-destructive/40 text-destructive">{error}</Alert>}
            <Button className="w-full" type="submit" disabled={!canSubmit}>
              {loading ? "Securing wallet..." : submitLabel}
            </Button>
          </form>
        </CardContent>
      </Card>
    </div>
  );
}
