import { useMemo, useState } from "react";
import { toast } from "sonner";

import { FormField } from "@/components/form/FormField";
import { Button } from "@/components/ui/button";
import { Card, CardContent, CardHeader, CardTitle } from "@/components/ui/card";
import { Input } from "@/components/ui/input";
import { cn } from "@/lib/utils";

import type { Patient } from "./patientStore";
import { usePatients } from "./usePatients";

function formatPatientName(patient: Patient) {
  return `${patient.lastName}, ${patient.firstName}`;
}

function PatientRow(props: { patient: Patient; selected: boolean; onSelect: () => void }) {
  return (
    <button
      type="button"
      onClick={props.onSelect}
      className={cn(
        "grid w-full gap-0.5 border-b border-border/70 px-3 py-2 text-left transition-colors hover:bg-muted/50",
        props.selected ? "bg-muted/60" : "bg-transparent",
      )}
    >
      <div className="flex items-center justify-between gap-3">
        <span className="text-sm font-medium">{formatPatientName(props.patient)}</span>
        <span className="text-xs text-muted-foreground">{props.patient.dateOfBirth}</span>
      </div>
      <div className="text-xs text-muted-foreground">{props.patient.phone ?? "—"}</div>
    </button>
  );
}

export function FrontDeskDashboard(props: { organizationId: string }) {
  const { patients, patientsById, addPatient, findPatients } = usePatients(props.organizationId);

  const [searchQuery, setSearchQuery] = useState("");
  const filteredPatients = useMemo(() => {
    const trimmed = searchQuery.trim();
    return trimmed ? findPatients(trimmed) : patients;
  }, [findPatients, patients, searchQuery]);

  const [selectedPatientId, setSelectedPatientId] = useState<string | null>(null);
  const selectedPatient = selectedPatientId ? patientsById.get(selectedPatientId) : undefined;

  const [firstName, setFirstName] = useState("");
  const [lastName, setLastName] = useState("");
  const [dateOfBirth, setDateOfBirth] = useState("");
  const [phone, setPhone] = useState("");
  const canRegister =
    firstName.trim().length > 0 && lastName.trim().length > 0 && dateOfBirth.length > 0;
  const formIsEmpty =
    firstName.length === 0 &&
    lastName.length === 0 &&
    dateOfBirth.length === 0 &&
    phone.length === 0;

  return (
    <div className="grid gap-6">
      <div>
        <h1 className="text-2xl font-semibold tracking-tight">Front desk</h1>
        <p className="mt-1 text-sm text-muted-foreground">
          Register patients and quickly find existing records (stored locally for now).
        </p>
      </div>

      <div className="grid gap-6 lg:grid-cols-5">
        <Card className="lg:col-span-2">
          <CardHeader>
            <CardTitle className="text-lg">Register patient</CardTitle>
          </CardHeader>
          <CardContent>
            <form
              className="grid gap-4"
              onSubmit={(e) => {
                e.preventDefault();
                if (!canRegister) return;

                const created = addPatient({
                  firstName,
                  lastName,
                  dateOfBirth,
                  phone: phone.trim() ? phone.trim() : undefined,
                });

                toast.success("Patient registered", {
                  description: formatPatientName(created),
                });

                setSelectedPatientId(created.id);
                setFirstName("");
                setLastName("");
                setDateOfBirth("");
                setPhone("");
              }}
            >
              <FormField id="first-name" label="First name">
                <Input
                  id="first-name"
                  value={firstName}
                  onChange={(e) => setFirstName(e.currentTarget.value)}
                  autoComplete="given-name"
                  placeholder="Ava"
                />
              </FormField>

              <FormField id="last-name" label="Last name">
                <Input
                  id="last-name"
                  value={lastName}
                  onChange={(e) => setLastName(e.currentTarget.value)}
                  autoComplete="family-name"
                  placeholder="Nguyen"
                />
              </FormField>

              <FormField id="dob" label="Date of birth">
                <Input
                  id="dob"
                  type="date"
                  value={dateOfBirth}
                  onChange={(e) => setDateOfBirth(e.currentTarget.value)}
                />
              </FormField>

              <FormField id="phone" label="Phone (optional)">
                <Input
                  id="phone"
                  value={phone}
                  onChange={(e) => setPhone(e.currentTarget.value)}
                  autoComplete="tel"
                  placeholder="+1 (555) 555-5555"
                />
              </FormField>

              <div className="flex items-center justify-between gap-3 pt-2">
                <Button
                  type="button"
                  variant="ghost"
                  disabled={formIsEmpty}
                  onClick={() => {
                    setFirstName("");
                    setLastName("");
                    setDateOfBirth("");
                    setPhone("");
                  }}
                >
                  Clear
                </Button>
                <Button type="submit" disabled={!canRegister}>
                  Register
                </Button>
              </div>
            </form>
          </CardContent>
        </Card>

        <Card className="lg:col-span-3">
          <CardHeader className="gap-3 sm:flex-row sm:items-center sm:justify-between">
            <CardTitle className="text-lg">Find patient</CardTitle>
            <div className="text-xs text-muted-foreground">
              {patients.length} total · {filteredPatients.length} shown
            </div>
          </CardHeader>
          <CardContent className="grid gap-4">
            <Input
              value={searchQuery}
              onChange={(e) => setSearchQuery(e.currentTarget.value)}
              placeholder="Search name, DOB, or phone…"
            />

            <div className="overflow-hidden rounded-md border">
              {filteredPatients.length === 0 ? (
                <div className="p-6 text-sm text-muted-foreground">
                  No patients match this search.
                </div>
              ) : (
                <div className="max-h-[360px] overflow-auto">
                  {filteredPatients.map((patient) => (
                    <PatientRow
                      key={patient.id}
                      patient={patient}
                      selected={patient.id === selectedPatientId}
                      onSelect={() => setSelectedPatientId(patient.id)}
                    />
                  ))}
                </div>
              )}
            </div>

            <div className="rounded-md border bg-muted/30 p-4">
              <div className="text-xs font-medium uppercase tracking-wider text-muted-foreground">
                Selected patient
              </div>
              {selectedPatient ? (
                <div className="mt-3 grid gap-2 text-sm">
                  <div className="flex items-baseline justify-between gap-4">
                    <span className="text-muted-foreground">Name</span>
                    <span className="font-medium">{formatPatientName(selectedPatient)}</span>
                  </div>
                  <div className="flex items-baseline justify-between gap-4">
                    <span className="text-muted-foreground">DOB</span>
                    <span className="font-mono text-xs">{selectedPatient.dateOfBirth}</span>
                  </div>
                  <div className="flex items-baseline justify-between gap-4">
                    <span className="text-muted-foreground">Phone</span>
                    <span className="font-mono text-xs">{selectedPatient.phone ?? "—"}</span>
                  </div>
                </div>
              ) : (
                <div className="mt-3 text-sm text-muted-foreground">
                  Select a patient from the list to see details.
                </div>
              )}
            </div>
          </CardContent>
        </Card>
      </div>
    </div>
  );
}
