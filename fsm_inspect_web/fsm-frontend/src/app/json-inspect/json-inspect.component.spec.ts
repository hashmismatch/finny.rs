import { async, ComponentFixture, TestBed } from '@angular/core/testing';

import { JsonInspectComponent } from './json-inspect.component';

describe('JsonInspectComponent', () => {
  let component: JsonInspectComponent;
  let fixture: ComponentFixture<JsonInspectComponent>;

  beforeEach(async(() => {
    TestBed.configureTestingModule({
      declarations: [ JsonInspectComponent ]
    })
    .compileComponents();
  }));

  beforeEach(() => {
    fixture = TestBed.createComponent(JsonInspectComponent);
    component = fixture.componentInstance;
    fixture.detectChanges();
  });

  it('should create', () => {
    expect(component).toBeTruthy();
  });
});
