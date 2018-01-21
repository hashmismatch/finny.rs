import { Component, OnInit, ElementRef, ViewChild, Input, SimpleChanges } from '@angular/core';
import JsonTreeView from 'json-tree-view';
import { OnChanges } from '@angular/core/src/metadata/lifecycle_hooks';

@Component({
  selector: 'app-json-inspect',
  templateUrl: './json-inspect.component.html',
  styleUrls: ['./json-inspect.component.scss']
})
export class JsonInspectComponent implements OnInit, OnChanges {
  ngOnChanges(changes: SimpleChanges): void {
    
    this.jsonView.value = this.data;
    this.jsonView.expand(true);
    this.jsonView.refresh();
    if (changes['data'].previousValue == null && changes['data'].currentValue != null) {
      this.ngAfterViewInit();
    } 
  }
  
  @ViewChild('root') root: ElementRef;

  @Input() data: any = null;

  constructor() {
    this.jsonView = new JsonTreeView("State", this.data, null, false);
    this.jsonView.withRootName = false;
    this.jsonView.readonly = true;
    this.jsonView.alwaysShowRoot = false;
  }

  ngOnInit() {
    
  }

  private jsonView: any;

  ngAfterViewInit() {     
    if (this.data != null) {
      if (this.root.nativeElement.childElementCount == 0) {
        this.root.nativeElement.appendChild(this.jsonView.dom);
      }
    }
  }

}
