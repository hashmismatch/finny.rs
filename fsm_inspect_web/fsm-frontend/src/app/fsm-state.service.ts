import { Injectable } from '@angular/core';
import { HttpClient } from '@angular/common/http';
import { FsmEvent, FsmEventStateTransitioned, FsmEventProcessingEvent, FsmEventState, FsmEventAction, FsmEventStateTransitionStart, parseTableId, ModifiedDataStructure, parseDataStructKind, DataStructKind, dataStructureIdToString } from './fsm-event';
import { BehaviorSubject } from 'rxjs/BehaviorSubject';
import { Observable } from 'rxjs/Observable';
import { Subject } from 'rxjs/Subject';

@Injectable()
export class FsmStateService {
  public fsmInfo: any;
  // hold the serialized state of structures
  private fsmStructuresData: Map<string, any> = new Map();
  
  private _fsmEvents: Array<FsmEvent> = new Array<FsmEvent>();
  public observableFsmEvents: Subject<FsmEvent> = new Subject();
  
  constructor(private http: HttpClient) {

    var host = 'localhost';
    //var host = '192.168.11.147';


    this.http.get('http://' + host + ':8002/fsm_info').subscribe(data => {
      this.fsmInfo = data;
    });


    const socket = new WebSocket('ws://' + host + ':9002');
    
        // Listen for messages
        socket.addEventListener('message', (event) => {
          
          var data = JSON.parse(event.data);

          var modifiedStructures = new Array<ModifiedDataStructure>();
          data.common.modified_structures.forEach(m => {
            var id = parseDataStructKind(m.id);
            var s = new ModifiedDataStructure(id, m.value);
            modifiedStructures.push(s);
          });          

          if (data.event.StateTransitionStart != null) {
            var st = data.event.StateTransitionStart;
            this.receivedFsmEvent(new FsmEventStateTransitionStart({
              id: data.common.id,
              fsm: data.common.fsm,
              transitionId: parseTableId(st.transition_id),
              region: st.region_id,
              from: st.from,
              to: st.to
            }), modifiedStructures);
          }

          if (data.event.StateTransitioned != null) {
    
            var st = data.event.StateTransitioned;
            this.receivedFsmEvent(new FsmEventStateTransitioned({
              id: data.common.id,
              fsm: data.common.fsm,
              transitionId: parseTableId(st.transition_id),
              region: st.region_id,
              from: st.from,
              to: st.to
            }), modifiedStructures);
          }
    
          if (data.event.ProcessingEvent != null) {
            var p = data.event.ProcessingEvent;
            this.receivedFsmEvent(new FsmEventProcessingEvent({
              id: data.common.id,
              fsm: data.common.fsm,
              eventKind: p.event_kind,
              eventData: p.event_data
            }), modifiedStructures);
          }
    
          if (data.event.StateEvent != null) {
            var p = data.event.StateEvent;
            this.receivedFsmEvent(new FsmEventState({
              id: data.common.id,
              fsm: data.common.fsm,
              transitionId: parseTableId(p.transition_id),
              stateEventKind: p.state_event_kind,
              stateName: p.state_name
            }), modifiedStructures);
          }
    
          if (data.event.Action != null) {
            var p = data.event.Action;
            this.receivedFsmEvent(new FsmEventAction({
              id: data.common.id,
              transitionId: parseTableId(p.transition_id),
              fsm: data.common.fsm,
              actionName: p.action_name
            }), modifiedStructures);
          }

          

        });    
  }

  public getFsmEvents(): Array<FsmEvent> {
    return this._fsmEvents;
  }

  public getEventCount(): number {
    return this._fsmEvents.length;
  }

  private receivedFsmEvent(ev: FsmEvent, modifiedStructures: Array<ModifiedDataStructure>) {
    this._fsmEvents.push(ev);
    this.observableFsmEvents.next(ev);
    //this._observableEvents.next(ev);

    var stateEvent = ev as FsmEventState;
    if (stateEvent != null) {
      var stateKey = stateEvent.fsm + "_" + stateEvent.stateName;
      /*
      if (stateEvent.stateData !== null) {
        this.fsmStructuresData.set(stateKey, stateEvent.stateData);
      }
      */

      if (stateEvent.stateEventKind == "Enter") {
        // missing region
      }
    }

    var transitionEvent = ev as FsmEventStateTransitioned;
    if (transitionEvent != null) {
      this.fsmActiveState.set(ev.fsm + "_" + transitionEvent.region, transitionEvent.to);
    }

    for (let m of modifiedStructures) {
      var key = ev.fsm + "_" + dataStructureIdToString(m.id);
      this.fsmStructuresData.set(key, m.value);
    }
  }


  public getStructData(fsm: string, id: DataStructKind): any {
    var key = fsm + "_" + dataStructureIdToString(id);
    return this.fsmStructuresData.get(key);
  }

  private fsmActiveState: Map<string, string> = new Map();
  public regionActiveState(fsm: string, region: number): string {
    return this.fsmActiveState.get(fsm + "_" + region);
  }
}