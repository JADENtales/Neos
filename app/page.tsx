"use client"
import styles from "./page.module.css";
import "bootstrap/dist/css/bootstrap.min.css";
import { useEffect, useRef, useState } from "react";
import { invoke } from '@tauri-apps/api/tauri'
import { listen } from '@tauri-apps/api/event'

export default function Home() {
  const names = ["全体", "一般", "耳打ち", "チーム", "クラブ", "システム", "叫び"];
  const colors = ["white", "white", "orange", "cyan", "violet", "yellow", "lightgreen"];
  const init = useRef(false);
  const textareaRefs = [
    useRef((null as unknown) as HTMLTextAreaElement),
    useRef((null as unknown) as HTMLTextAreaElement),
    useRef((null as unknown) as HTMLTextAreaElement),
    useRef((null as unknown) as HTMLTextAreaElement),
    useRef((null as unknown) as HTMLTextAreaElement),
    useRef((null as unknown) as HTMLTextAreaElement),
    useRef((null as unknown) as HTMLTextAreaElement),
  ];
  const labelRefs = [
    useRef((null as unknown) as HTMLLabelElement),
    useRef((null as unknown) as HTMLLabelElement),
    useRef((null as unknown) as HTMLLabelElement),
    useRef((null as unknown) as HTMLLabelElement),
    useRef((null as unknown) as HTMLLabelElement),
    useRef((null as unknown) as HTMLLabelElement),
    useRef((null as unknown) as HTMLLabelElement),
  ];
  const spacerRef = useRef((null as unknown) as HTMLDivElement);
  const [messages, setMessages] = useState([...Array(names.length)].map(_ => ""));
  const [views, setViews] = useState([...Array(names.length)].map(_ => true));
  const [verbose, setVerbose] = useState(false);
  const [wrap, setWrap] = useState("soft");
  const [vertical, setVertical] = useState(true);
  const [autoScroll, setAutoScroll] = useState(true);
  type State = { verbose: boolean, wrap: string, vertical: boolean, auto_scroll: boolean };

  useEffect(() => {
    const resizeTextareaImpl = () => {
      const view_count = views.filter((e: any) => e).length;
      for (let i = 0; i < views.length; ++i) {
        if (!views[i]) {
          continue;
        }
        const height = (window.innerHeight - labelRefs[i].current.offsetHeight * view_count - spacerRef.current.offsetHeight) / view_count;
        textareaRefs[i].current.style.height = height + "px";
      }
    };
    const resizeTextarea = (event: Event) => resizeTextareaImpl();
    resizeTextareaImpl();
    window.addEventListener("resize", resizeTextarea);

    const f = async () => {
      for (let i = 0; i < names.length; ++i) {
        await listen('view' + i, async event => {
          const views = await invoke("get_views") as [boolean, boolean, boolean, boolean, boolean, boolean, boolean];
          setViews(views);
        });
      }
      await listen('verbose', async event => {
        const state = await invoke("get_state") as State;
        setVerbose(state.verbose);
      });
      await listen('wrap', async event => {
        const state = await invoke("get_state") as State;
        setWrap(state.wrap);
      });
      await listen('vertical', async event => {
        const state = await invoke("get_state") as State;
        setVertical(state.vertical);
      });
      await listen('auto_scroll', async event => {
        const state = await invoke("get_state") as State;
        setAutoScroll(state.auto_scroll);
      });
      const views = await invoke("get_views") as [boolean, boolean, boolean, boolean, boolean, boolean, boolean];
      setViews(views);
      const state = await invoke("get_state") as State;
      setVerbose(state.verbose);
      setWrap(state.wrap);
      setVertical(state.vertical);
      setAutoScroll(state.auto_scroll);
    };
    if (!init.current) {
      console.log('init')
      init.current = true;
      f();
    }

    const id = setInterval(async () => {
      const msgs = await invoke("read_log") as [string, string, string][][];
      for (let i = 0; i < msgs.length; ++i) {
        const msg = msgs[i].map(e => {
          if (verbose) {
            return e[2] + " " + e[0];
          }
          return e[0];
        }).join("\n");
        setMessages(prev => prev.map((e, j) => i === j ? msg : e));
        if (!autoScroll) {
          continue;
        }
        if (textareaRefs[i].current !== null) {
          textareaRefs[i].current.scrollTop = textareaRefs[i].current.scrollHeight;
        }
      }
    }, 100);

    return () => {
      console.log('clear');
      clearInterval(id);
      window.removeEventListener("resize", resizeTextarea);
    };
  }, [verbose, autoScroll, views]);

  return (
    <div className="container-fluid">
      {vertical && names.map((name, i) => {
        return views[i] && (
          <div key={name}>
            <label className={`pt-2 form-label ${styles.label}`} ref={labelRefs[i]}>{name}</label>
            <textarea className={"form-control " + styles.textarea} style={{"color": colors[i]}} value={messages[i]} rows={3} onChange={_ => {}} wrap={wrap} readOnly ref={textareaRefs[i]}></textarea>
          </div>
        );
      })}
      { vertical && <div className="invisible p-1" ref={spacerRef}></div>}
      {!vertical &&
        <div className="row">
          {names.map((name, i) => {
            return views[i] && (
              <div className="col" key={name}>
                <label className="form-label" ref={labelRefs[i]}>{name}</label>
                <textarea className={`form-control ${styles.textarea} ${styles.horizontal}`} style={{"color": colors[i]}} value={messages[i]} rows={3} onChange={_ => {}} wrap={wrap} readOnly ref={textareaRefs[i]}></textarea>
              </div>
            );
          })}
        </div>
      }
    </div>
  );
}
