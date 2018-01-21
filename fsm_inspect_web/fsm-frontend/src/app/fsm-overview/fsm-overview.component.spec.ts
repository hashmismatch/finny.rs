import { async, ComponentFixture, TestBed } from '@angular/core/testing';

import { FsmOverviewComponent } from './fsm-overview.component';

describe('FsmOverviewComponent', () => {
  let component: FsmOverviewComponent;
  let fixture: ComponentFixture<FsmOverviewComponent>;

  beforeEach(async(() => {
    TestBed.configureTestingModule({
      declarations: [ FsmOverviewComponent ]
    })
    .compileComponents();
  }));

  beforeEach(() => {
    fixture = TestBed.createComponent(FsmOverviewComponent);
    component = fixture.componentInstance;
    fixture.detectChanges();
  });

  it('should create', () => {
    expect(component).toBeTruthy();
  });
});
