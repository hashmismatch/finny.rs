import { TestBed, inject } from '@angular/core/testing';

import { FsmStateService } from './fsm-state.service';

describe('FsmStateService', () => {
  beforeEach(() => {
    TestBed.configureTestingModule({
      providers: [FsmStateService]
    });
  });

  it('should be created', inject([FsmStateService], (service: FsmStateService) => {
    expect(service).toBeTruthy();
  }));
});
