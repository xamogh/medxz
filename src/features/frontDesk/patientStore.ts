export type Patient = {
  id: string;
  organizationId: string;
  firstName: string;
  lastName: string;
  dateOfBirth: string;
  phone: string | null;
  createdAt: string;
};

export type NewPatientInput = {
  firstName: string;
  lastName: string;
  dateOfBirth: string;
  phone?: string;
};

const STORAGE_PREFIX = "medxz.patients.v1";

function storageKey(organizationId: string) {
  return `${STORAGE_PREFIX}.${organizationId}`;
}

export function loadPatients(organizationId: string): Patient[] {
  const raw = localStorage.getItem(storageKey(organizationId));
  if (!raw) return [];

  try {
    const parsed = JSON.parse(raw) as unknown;
    if (!Array.isArray(parsed)) return [];
    return parsed.filter(isPatient);
  } catch {
    return [];
  }
}

export function savePatients(organizationId: string, patients: Patient[]) {
  localStorage.setItem(storageKey(organizationId), JSON.stringify(patients));
}

export function createPatient(organizationId: string, input: NewPatientInput): Patient {
  const firstName = input.firstName.trim();
  const lastName = input.lastName.trim();
  const dateOfBirth = input.dateOfBirth.trim();

  return {
    id: crypto.randomUUID(),
    organizationId,
    firstName,
    lastName,
    dateOfBirth,
    phone: input.phone?.trim() ? input.phone.trim() : null,
    createdAt: new Date().toISOString(),
  };
}

export function patientMatchesQuery(patient: Patient, query: string): boolean {
  const q = query.trim().toLowerCase();
  if (!q) return true;

  const normalizedPhoneQuery = q.replace(/[^\d+]/g, "");
  const normalizedPhone = (patient.phone ?? "").replace(/[^\d+]/g, "");

  const haystack = [
    patient.firstName,
    patient.lastName,
    `${patient.firstName} ${patient.lastName}`,
    patient.dateOfBirth,
    patient.phone ?? "",
  ]
    .join(" ")
    .toLowerCase();

  return (
    haystack.includes(q) ||
    (normalizedPhoneQuery.length > 0 && normalizedPhone.includes(normalizedPhoneQuery))
  );
}

function isPatient(value: unknown): value is Patient {
  if (!value || typeof value !== "object") return false;
  const record = value as Record<string, unknown>;
  return (
    typeof record.id === "string" &&
    typeof record.organizationId === "string" &&
    typeof record.firstName === "string" &&
    typeof record.lastName === "string" &&
    typeof record.dateOfBirth === "string" &&
    typeof record.createdAt === "string"
  );
}
