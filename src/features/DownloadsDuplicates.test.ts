import { render, screen, fireEvent, waitFor } from '@testing-library/svelte';
import { describe, it, expect, vi } from 'vitest';
import { createAppStore } from '../state';
import { fakeBackend, session } from '../testing/fake-backend';
import Duplicates from './Duplicates.svelte';
import RecordDetails from '../components/RecordDetails.svelte';
import type { DuplicateReport } from '../bindings/DuplicateReport';

const report: DuplicateReport = {
  id:'downloads-fixture', sessionId:'fixture', createdAt:'2026-10-27T12:00:00Z', status:'partial', stage:'Content analysis finished',
  examined:6, total:6, hashedBytes:60, photoMatches:[], gaps:['Illustrative bounded source'], qualification:'Illustrative test fixture, not device measurements.',
  request:{ sessionId:'fixture',location:'downloads',similarPhotos:false }, scopeRoot:'/Users/fixture/Downloads',
  groups:[
    {id:'old', digest:'old-fixture',files:[
      {path:'/Users/fixture/Downloads/old-copy',logicalBytes:10,allocatedBytes:4096,modifiedAt:'2026-01-01T12:00:00Z',reviewSupported:true,reviewQualification:'Fixture copy'},
      {path:'/Users/fixture/Downloads/recent-keeper',logicalBytes:10,allocatedBytes:4096,modifiedAt:'2026-10-26T12:00:00Z',reviewSupported:true,reviewQualification:'Fixture copy'},
    ]},
    {id:'new', digest:'new-fixture',files:[
      {path:'/Users/fixture/Downloads/new-copy',logicalBytes:20,allocatedBytes:4096,modifiedAt:'2026-10-25T12:00:00Z',reviewSupported:true,reviewQualification:'Fixture copy'},
      {path:'/Users/fixture/Downloads/new-keeper',logicalBytes:20,allocatedBytes:4096,modifiedAt:'2026-10-25T12:00:00Z',reviewSupported:true,reviewQualification:'Fixture copy'},
    ]},
    {id:'unknown', digest:'unknown-fixture',files:[
      {path:'/Users/fixture/Downloads/unknown-copy',logicalBytes:30,allocatedBytes:4096,modifiedAt:'',reviewSupported:true,reviewQualification:'Fixture copy'},
      {path:'/Users/fixture/Downloads/unknown-keeper',logicalBytes:30,allocatedBytes:4096,modifiedAt:'',reviewSupported:true,reviewQualification:'Fixture copy'},
    ]},
  ],
};
async function setup(saved: DuplicateReport | null = report) {
  const service = fakeBackend({
    history:async () => [{...session,status:'complete'}],
    records:vi.fn(async () => saved ? [{id:saved.id,kind:'duplicates',createdAt:saved.createdAt,status:saved.status,title:'Fixture duplicate report'}] : []),
    record:vi.fn(async () => ({kind:'duplicates' as const,data:saved!})),
    analyzeDuplicateScope:vi.fn(async (_request, onEvent) => {onEvent(report); return report;}),
  });
  const controller = createAppStore(service); await controller.initialize();
  return {controller,service};
}

describe('Downloads content review', () => {
  it('starts no content reads until Analyze, defaults to Downloads and requires a separate photo choice', async () => {
    const {controller,service}=await setup(null);
    render(Duplicates,{controller,onReview:vi.fn()});
    const analyze=screen.getByRole('button',{name:'Analyze contents'});
    await waitFor(() => expect(analyze).toBeEnabled());
    expect(screen.getByRole('combobox',{name:'Content-check location'})).toHaveValue('downloads');
    const photos=screen.getByRole('checkbox',{name:'Include similar PNG/JPEG photos in the next check'});
    expect(photos).not.toBeChecked();
    expect(service.analyzeDuplicateScope).not.toHaveBeenCalled();
    await fireEvent.click(analyze);
    await waitFor(() => expect(service.analyzeDuplicateScope).toHaveBeenCalledWith({sessionId:'fixture',location:'downloads',similarPhotos:false},expect.any(Function)));
    expect(service.analyzeDuplicates).not.toHaveBeenCalled();
    expect(service.executeCleanup).not.toHaveBeenCalled();
    await controller.dispose();
  });

  it('filters retained groups by modification age while keeping the newer keeper visible and selections empty', async () => {
    const {controller,service}=await setup(); render(Duplicates,{controller,onReview:vi.fn()});
    await screen.findByText('Location: Downloads only.',{exact:false});
    expect(screen.getAllByRole('checkbox',{name:/Review removal/}).every((input) => !(input as HTMLInputElement).checked)).toBe(true);
    await fireEvent.change(screen.getByRole('combobox',{name:'Last modified'}),{target:{value:'90'}});
    expect(screen.getByRole('radio',{name:'Keep /Users/fixture/Downloads/recent-keeper'})).toBeInTheDocument();
    expect(screen.queryByRole('radio',{name:'Keep /Users/fixture/Downloads/new-keeper'})).not.toBeInTheDocument();
    await fireEvent.click(screen.getByRole('radio',{name:'Keep /Users/fixture/Downloads/recent-keeper'}));
    await fireEvent.click(screen.getByRole('checkbox',{name:'Review removal of /Users/fixture/Downloads/old-copy'}));
    expect(screen.getByRole('button',{name:'Review selected copies'})).toBeEnabled();
    await fireEvent.input(screen.getByRole('searchbox',{name:'Find a path'}),{target:{value:'old-copy'}});
    expect(screen.getByRole('radio',{name:'Keep /Users/fixture/Downloads/recent-keeper'})).not.toBeChecked();
    expect(screen.getByRole('button',{name:'Review selected copies'})).toBeDisabled();
    expect(service.analyzeDuplicateScope).not.toHaveBeenCalled();
    expect(service.previewDuplicates).not.toHaveBeenCalled();
    expect(service.records).toHaveBeenCalledTimes(1);
    await controller.dispose();
  });

  it('distinguishes unknown age and filter emptiness from absent exact copies', async () => {
    const {controller,service}=await setup(); render(Duplicates,{controller,onReview:vi.fn()});
    await screen.findByRole('combobox',{name:'Last modified'});
    await fireEvent.change(screen.getByRole('combobox',{name:'Last modified'}),{target:{value:'unknown'}});
    expect(screen.getByRole('radio',{name:'Keep /Users/fixture/Downloads/unknown-copy'})).toBeInTheDocument();
    expect(screen.queryByRole('radio',{name:'Keep /Users/fixture/Downloads/old-copy'})).not.toBeInTheDocument();
    await fireEvent.input(screen.getByRole('searchbox',{name:'Find a path'}),{target:{value:'absent'}});
    expect(screen.getByRole('heading',{name:'No groups match these filters.'})).toBeInTheDocument();
    expect(screen.queryByText('No exact groups confirmed.')).not.toBeInTheDocument();
    await fireEvent.click(screen.getByRole('button',{name:'Clear filters'}));
    expect(screen.getAllByRole('radio')).toHaveLength(6);
    expect(service.analyzeDuplicateScope).not.toHaveBeenCalled();
    await controller.dispose();
  });

  it('requires a new analysis when changing source and passes explicit photo consent', async () => {
    const {controller,service}=await setup(); render(Duplicates,{controller,onReview:vi.fn()});
    await screen.findByRole('radio',{name:'Keep /Users/fixture/Downloads/old-copy'});
    await fireEvent.change(screen.getByRole('combobox',{name:'Content-check location'}),{target:{value:'indexed'}});
    expect(screen.queryAllByRole('radio')).toHaveLength(0);
    expect(service.analyzeDuplicateScope).not.toHaveBeenCalled();
    await fireEvent.click(screen.getByRole('checkbox',{name:'Include similar PNG/JPEG photos in the next check'}));
    await fireEvent.click(screen.getByRole('button',{name:'Analyze contents'}));
    await waitFor(() => expect(service.analyzeDuplicateScope).toHaveBeenCalledWith({sessionId:'fixture',location:'indexed',similarPhotos:true},expect.any(Function)));
    await controller.dispose();
  });

  it('keeps legacy History scope unrecorded and reopening remains a read', async () => {
    const {controller,service}=await setup(null);
    render(RecordDetails,{controller,record:{kind:'duplicates',data:{...report,request:null,scopeRoot:null}}});
    expect(screen.getByText(/Historical analysis choices were not recorded/)).toBeInTheDocument();
    expect(service.analyzeDuplicateScope).not.toHaveBeenCalled();
    expect(service.analyzeDuplicates).not.toHaveBeenCalled();
    await controller.dispose();
  });

  it('reaches the sole matching copy after member 100 and retains an explicitly chosen keeper across pages', async () => {
    const files=Array.from({length:101},(_,index)=>({...report.groups[0].files[0],path:`/Users/fixture/Downloads/copy-${index}`,modifiedAt:index===100?'2026-01-01T12:00:00Z':'2026-10-26T12:00:00Z'}));
    const large={...report,groups:[{id:'large',digest:'fixture',files}]};
    const {controller,service}=await setup(large);render(Duplicates,{controller,onReview:vi.fn()});
    await screen.findByRole('combobox',{name:'Last modified'});
    await fireEvent.input(screen.getByRole('searchbox',{name:'Find a path'}),{target:{value:'copy-100'}});
    await fireEvent.change(screen.getByRole('combobox',{name:'Last modified'}),{target:{value:'90'}});
    expect(screen.getAllByText('Copies 1–100 of 101')).toHaveLength(2);
    await fireEvent.click(screen.getByRole('radio',{name:'Keep /Users/fixture/Downloads/copy-0'}));
    await fireEvent.click(screen.getAllByRole('button',{name:'Next copies'})[0]);
    expect(screen.getByRole('radio',{name:'Keep /Users/fixture/Downloads/copy-100'})).toBeInTheDocument();
    expect(screen.getByText('/Users/fixture/Downloads/copy-0')).toBeInTheDocument();
    await fireEvent.click(screen.getByRole('checkbox',{name:'Review removal of /Users/fixture/Downloads/copy-100'}));
    await fireEvent.click(screen.getByRole('button',{name:'Review selected copies'}));
    expect(service.previewDuplicates).toHaveBeenCalledWith({reportId:large.id,groupId:'large',keepPath:'/Users/fixture/Downloads/copy-0',removePaths:['/Users/fixture/Downloads/copy-100']});
    expect(service.analyzeDuplicateScope).not.toHaveBeenCalled();
    expect(service.executeCleanup).not.toHaveBeenCalled();
    await fireEvent.change(screen.getByRole('combobox',{name:'Last modified'}),{target:{value:'any'}});
    expect(screen.getAllByText('Copies 1–100 of 101')).toHaveLength(2);
    expect(screen.getByRole('radio',{name:'Keep /Users/fixture/Downloads/copy-0'})).not.toBeChecked();
    await controller.dispose();
  });
});
