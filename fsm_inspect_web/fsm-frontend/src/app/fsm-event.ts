
export abstract class FsmEvent {
    id: number;
    fsm: string;

    public constructor(init?:Partial<FsmEvent>) {
        Object.assign(this, init);
    }    
}

export class FsmEventStateTransitionStart extends FsmEvent {
    region: number;
    from: string;
    to: string;
    transitionId: TransitionId;

    public constructor(init?:Partial<FsmEventStateTransitionStart>) {
        super(init)
        Object.assign(this, init);
    }
}

export class FsmEventStateTransitioned extends FsmEvent {
    region: number;
    from: string;
    to: string;
    transitionId: TransitionId;

    public constructor(init?:Partial<FsmEventStateTransitioned>) {
        super(init)
        Object.assign(this, init);
    }
}

export class FsmEventActionExecuted extends FsmEvent {
    region: number;
    action: string;
    transitionId: TransitionId;

    public constructor(init?:Partial<FsmEventActionExecuted>) {
        super(init)
        Object.assign(this, init);
    }    
}

export class FsmEventProcessingEvent extends FsmEvent {
    eventKind: string
    eventData: any;

    public constructor(init?:Partial<FsmEventProcessingEvent>) {
        super(init)
        Object.assign(this, init);
    }    
}


export class FsmEventState extends FsmEvent {
    stateEventKind: string
    stateName: string;
    transitionId: TransitionId;


    public constructor(init?:Partial<FsmEventState>) {
        super(init)
        Object.assign(this, init);
    }    
}


export class FsmEventAction extends FsmEvent {
    actionName: String;
    transitionId: TransitionId;

    public constructor(init?:Partial<FsmEventAction>) {
        super(init)
        Object.assign(this, init);
    }    
}

export type TransitionId = TransitionKind | TransitionTableId;
export class TransitionTableId {
    public constructor(public id: number) { }
}
export enum TransitionKind {
    Start,
    Stop
}

export function parseTableId(source: any): TransitionId {
    if (source.Table != null) {
        return new TransitionTableId(source.Table);
    } else if (source.Start != null) {
        return TransitionKind.Start;
    } else if (source.Stop != null) {
        return TransitionKind.Stop;
    } else {
        return null;
    }
}


export type DataStructKind = DataStructKindEnum | DataStructKindState;

export enum DataStructKindEnum {
    Context
}

export class DataStructKindState {
    public constructor(public state: string) { }
}

export function parseDataStructKind(ds: any): DataStructKind {
    if (ds == "Context") {
        return DataStructKindEnum.Context;
    } else if (ds.State != null) {
        return new DataStructKindState(ds.State);
    } else {
        return null;
    }
}

export class ModifiedDataStructure {
    public constructor(public id: DataStructKind, public value: any) { }
}

export function dataStructureIdToString(dsKind: DataStructKind): string {
    if (dsKind == DataStructKindEnum.Context) {
        return "context";
    }

    var state = dsKind as DataStructKindState;
    if (state != null) {
        return "state_" + state.state;
    }
    
    return null;
}