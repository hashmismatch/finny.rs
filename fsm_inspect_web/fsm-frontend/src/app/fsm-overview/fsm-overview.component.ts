import { Component, OnInit, ElementRef, ViewChild } from '@angular/core';
import { HttpClient } from '@angular/common/http';
import { FsmStateService } from '../fsm-state.service';
import { DataStructKindState, DataStructKindEnum } from '../fsm-event';

import JsonTreeView from 'json-tree-view';

@Component({
  selector: 'app-fsm-overview',
  templateUrl: './fsm-overview.component.html',
  styleUrls: ['./fsm-overview.component.scss']
})
export class FsmOverviewComponent implements OnInit {

  constructor(private fsmState: FsmStateService) { }
  ngOnInit() {
    
  }
  

  getStateData(fsm: string, state: string): any {
    return this.fsmState.getStructData(fsm, new DataStructKindState(state));
  }

  getContextData(fsm: string): any {
    return this.fsmState.getStructData(fsm, DataStructKindEnum.Context);
  }

}
