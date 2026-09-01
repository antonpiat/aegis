import { useNavigate } from "react-router-dom";
import { FileKey2, KeyRound, Usb, FileText } from "lucide-react";
import { Button } from "@/components/ui/button";
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from "@/components/ui/card";

export function RestoreChoicePage() {
  const navigate = useNavigate();

  return (
    <div className="flex min-h-screen items-center justify-center bg-background p-6">
      <Card className="w-full max-w-md">
        <CardHeader>
          <CardTitle>Import wallet</CardTitle>
          <CardDescription>Choose how you want to restore access.</CardDescription>
        </CardHeader>
        <CardContent className="space-y-2">
          <Button className="w-full justify-start" onClick={() => navigate("/onboarding/import")}>
            <FileText className="h-4 w-4" />
            Import Recovery Phrase
          </Button>
          <Button
            className="w-full justify-start"
            variant="outline"
            onClick={() => navigate("/onboarding/import-key")}
          >
            <KeyRound className="h-4 w-4" />
            Import Private Key
          </Button>
          <Button className="w-full justify-start" variant="outline" disabled>
            <Usb className="h-4 w-4" />
            Connect hardware wallet
            <span className="ml-auto text-xs font-normal text-muted-foreground">Coming soon</span>
          </Button>
          <Button
            className="w-full justify-start"
            variant="ghost"
            onClick={() => navigate("/onboarding/import-backup")}
          >
            <FileKey2 className="h-4 w-4" />
            Import from backup
          </Button>
          <Button variant="outline" className="w-full" onClick={() => navigate("/onboarding")}>
            Back
          </Button>
        </CardContent>
      </Card>
    </div>
  );
}
