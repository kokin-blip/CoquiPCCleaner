import { render, screen, fireEvent, waitFor } from '@testing-library/svelte';
import { describe, it, expect } from 'vitest';
import { createAppStore } from '../state';
import { fakeBackend } from '../testing/fake-backend';
import type { AdviceReport } from '../bindings/AdviceReport';
import AdviceContext from './AdviceContext.svelte';
import RecordDetails from './RecordDetails.svelte';

const url = 'https://support.apple.com/macos/upgrade';
const report: AdviceReport = {
  id: 'fixture-advice',
  createdAt: '2030-01-02T12:00:00Z',
  workload: 'Fixture',
  assessment: 'Fixture assessment',
  findings: [],
  scenarios: [],
  sources: [url],
  qualification: 'Illustrative test data',
  sourceReviews: [
    {
      url,
      publisher: 'Apple',
      title: 'macOS Ventura upgrade guide',
      reviewedOn: '2026-10-23',
      scope: 'Ventura-specific guidance; current eligibility is not verified.',
    },
  ],
  verificationLimits: [
    'Exact application and peripheral requirements are not checked.',
  ],
};

describe('advice source provenance', () => {
  it('shows the retained vendor/date and limits, and opens a reference only on request', async () => {
    const service = fakeBackend();
    const controller = createAppStore(service);
    render(AdviceContext, { report, controller });
    expect(
      screen.getByText('Apple · Reviewed for this build: 2026-10-23'),
    ).toBeInTheDocument();
    expect(screen.getByText(report.verificationLimits[0])).toBeInTheDocument();
    expect(screen.getByText(report.sourceReviews[0].scope)).toBeInTheDocument();
    expect(
      screen.getByText(/does not fetch these pages or recheck compatibility/),
    ).toBeInTheDocument();
    expect(service.reference).not.toHaveBeenCalled();
    expect(service.assess).not.toHaveBeenCalled();
    await fireEvent.click(
      screen.getByRole('link', { name: 'macOS Ventura upgrade guide' }),
    );
    await waitFor(() => expect(service.reference).toHaveBeenCalledWith(url));
  });
  it('does not use a legacy report’s creation date as a source check', () => {
    render(AdviceContext, {
      report: { ...report, sourceReviews: [], verificationLimits: [] },
      controller: createAppStore(fakeBackend()),
    });
    expect(
      screen.getByText('Publisher not recorded · Review date not recorded'),
    ).toBeInTheDocument();
    expect(
      screen.getByText(/Structured verification limits were not retained/),
    ).toBeInTheDocument();
    expect(screen.queryByText(/2030-01-02/)).toBeNull();
    expect(screen.queryByText(/2026-10-23/)).toBeNull();
  });
  it('keeps the same provenance visible when reopening retained advice', () => {
    render(RecordDetails, {
      record: { kind: 'advice', data: report },
      controller: createAppStore(fakeBackend()),
    });
    expect(
      screen.getByRole('heading', { name: 'Fixture assessment' }),
    ).toBeInTheDocument();
    expect(
      screen.getByText('Apple · Reviewed for this build: 2026-10-23'),
    ).toBeInTheDocument();
    expect(screen.getByText(report.verificationLimits[0])).toBeInTheDocument();
  });
});
