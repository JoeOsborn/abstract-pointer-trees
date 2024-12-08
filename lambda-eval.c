#include <stdio.h>
#include <stdlib.h>

// lambda expressions
typedef enum { 
  APP,    // e1 (e2)
  LAM ,   // \x. e
  PTR ,   // (x) (in body of a fn)
  BASE    // constant
} expr_tag;

// constants (base type data, represented as strings)
struct base {
  char* data;
};

struct base b0 = { "zero" };
struct base b1 = { "one" };
struct base unit = {"()"};

// abstract binding trees
struct expr {
  expr_tag tag;
  union {
    struct expr** p;    // tag=PTR
    struct app {        // tag=APP
      struct expr* fun;
      struct expr* arg;
    } a;
    struct lam {        // tag=LAM
      struct expr** var;
      struct expr* body;
    } l;
    struct base* b;     // tag=BASE
  } data;
};


// pretty printing
char* tag2s(expr_tag c) {
  switch(c) {
  case APP: return "APP";
  case LAM: return "LAM";
  case PTR: return "PTR";
  case BASE: return "BASE";
  }
}

void indent(int n) {
  // printf("\n-- indenting %d tabs\n", n);
  for(int i=0; i < n; i++) {
    printf("\t");
  }
}

// TODO currently prints, want to produce string
// need sprintf and dynamic allocation or sthg
void expr2s(struct expr* e, int tabs) {
  printf("\n");
  indent(tabs);
  printf("@%p: %s { ", e, tag2s(e->tag));

  switch(e->tag) {
    case LAM: {
                printf("\n");
                indent(tabs+1);
                printf("var: %p,\n", e->data.l.var);
                indent(tabs+1);
                printf("body: ");
                expr2s(e->data.l.body, tabs+1);
                indent(tabs);
                printf("}\n ");
                return;
              }
    case APP: {
                printf("\n");
                indent(tabs+1);
                printf("fun: ");
                expr2s(e->data.a.fun, tabs+1);
                indent(tabs+1);
                printf("arg: ");
                expr2s(e->data.a.arg, tabs+1);
                indent(tabs);
                printf("}\n");
                return;
              }
    case PTR: {
                // indent(tabs);
                printf("%p -> ", e->data.p);
                // if we point to something,
                // recursively print it
                if(*(e->data.p)) {
                  expr2s(*(e->data.p), tabs+1);
                  indent(tabs);
                  printf("}\n");
                } else {
                  printf("0 }\n");
                }
                return;
              }
    case BASE: {
                 // indent(tabs);
                 printf("%s }\n", e->data.b->data);
               }
  }

  // return s;
}

/* Evaluation */

// not actually used, but isolates core logic
// for beta reduction
struct expr* step_app_lam(struct lam* l, struct expr* arg) {
  *(l->var) = arg;
  return (l->body);
}

struct expr* step(struct expr* e){
  // lambdas, constants, and null ptrs (vars) are values.
  // apps take steps. ptrs to things step to those things.
  switch(e->tag) {
    case LAM: return NULL;
    case PTR: {
      if(*(e->data.p)) {
        return(*(e->data.p));
      }
      return NULL;
    }
    case BASE: return NULL;
    case APP: {
      struct expr* f2 = step(e->data.a.fun);
      if(f2) { // function takes a step
        e->data.a.fun = f2;
        return e;
      } else { 
        struct expr* arg2 = step(e->data.a.arg);
        if(arg2) { // arg takes a step
          e->data.a.arg = arg2;
          return e;
        } else { // hopefully, beta reduction
          if(e->data.a.fun->tag == LAM) {
            struct lam l = e->data.a.fun->data.l;
            *(l.var) = e->data.a.arg;
            return (l.body);
          } else { // stuck state
            return NULL;
          } // function isn't lam
        } // arg is value
      } // function is value 
    } // end app case
  } // end switch
}

struct expr* eval(struct expr* e) {
  struct expr* e2 = e; // current value
  struct expr* e3 = e; // next value
  int iters = 0;

  printf("\nIteration 0:");
  while(1) {
    iters++;
    printf("evaluating expression ");
    expr2s(e2,0);
    e3 = step(e2); // get next value
    if (e3) { // if not null, not done yet
      printf("\nIteration %d: ", iters);
      e2 = e3; // so set current to next and continue
    } else { // if null, then current value is final
      printf("done\n");
      return e2;
    }
  }
}

// TODO: construct lambdas from more readable notation?

// \x.\f. f x
// lam { var:0, body { lam { var:1, app {fn: ref 1, arg {ref 0}}}}}

/* Util functions for constructing exprs */
struct expr makeApp(struct expr* f, struct expr* arg) {
  struct app a;
  a.fun = f;
  a.arg = arg;

  struct expr e;
  e.tag = APP;
  e.data.a = a;

  return e;
}

struct expr makeLam(struct expr** var, struct expr* body) {
  // printf("makin a lam\n");
  struct lam l;

  l.var = var;
  l.body = body;

  struct expr e;
  e.tag = LAM;
  e.data.l = l;

  return e;
}

/* Tests */

// lambda x. x
struct expr makeIdFn() {
  struct expr** var = malloc(sizeof(struct expr*));
  *var = NULL; // bound var doesn't point to anything
  struct expr* body = malloc(sizeof(struct expr));
  body->tag = PTR;
  body->data.p = var;
  return makeLam(var, body);
}

// lambda x. ()
struct expr makeConstFn() {
  struct expr** var = malloc(sizeof(struct expr*));
  *var = NULL;
  struct base* u = &unit;
  struct expr* body = malloc(sizeof(struct expr));
  body->tag = BASE;
  body->data.b = u;
  return makeLam(var, body);
}

// \x.\y. x
struct expr makeLamTrue() {
  struct expr** x = malloc(sizeof(struct expr*));
  *x = NULL;
  struct expr** y = malloc(sizeof(struct expr*));
  *y = NULL;
  struct expr* inner = malloc(sizeof(struct expr));
  inner->tag = PTR;
  inner->data.p = x;
  struct expr* outer = malloc(sizeof(struct expr));
  *outer = makeLam(y, inner);
  return makeLam(x, outer);
}

// \x.\y. y
struct expr makeLamFalse() {
  struct expr** x = malloc(sizeof(struct expr*));
  *x = NULL;
  struct expr** y = malloc(sizeof(struct expr*));
  *y = NULL;
  struct expr* inner = malloc(sizeof(struct expr));
  inner->tag = PTR;
  inner->data.p = y;
  struct expr* outer = malloc(sizeof(struct expr));
  *outer = makeLam(y, inner);
  return makeLam(x, outer);
}



// application of id to const
void test1() {
  struct expr lam_id = makeIdFn();
  struct expr lam_const = makeConstFn();
  // printf("%s\n", expr2s(&lam_id));
  printf("identity fn:\n");
  expr2s(&lam_id,0);

  printf("const fn:\n");
  expr2s(&lam_const,0);

  printf("application of id to const:\n");
  struct expr app_id_const = makeApp(&lam_id, &lam_const);
  expr2s(&app_id_const,0);

  // evaluation
  printf("------------------------------\n");
  printf("Evaluating id applied to const\n");

  struct expr* eval_result = eval(&app_id_const);
  printf("result of eval:\n");
  expr2s(eval_result,0);
  printf("\n");
}

// application of true to id
void test2() {
  printf("------------------------------\n");
  printf("Evaluating true applied to id\n");
  
  struct expr lam_id = makeIdFn();
  struct expr lam_true = makeLamTrue();

  struct expr app_true_id = makeApp(&lam_true, &lam_id);

  printf("Constructed expression:");
  expr2s(&app_true_id,0);

  struct expr* eval_result = eval(&app_true_id);
  printf("Result of eval:\n");
  expr2s(eval_result,0);
  printf("\n");

}

// application of false to id
void test3() {
  printf("------------------------------\n");
  printf("Evaluating false applied to id\n");
  
  struct expr lam_id = makeIdFn();
  struct expr lam_false = makeLamFalse();

  struct expr app_false_id = makeApp(&lam_false, &lam_id);

  printf("Constructed expression:");
  expr2s(&app_false_id,0);

  struct expr* eval_result = eval(&app_false_id);
  printf("Result of eval:\n");
  expr2s(eval_result,0);
  printf("\n");

}

int main(int argc, char** argv) {
  test2();
  return 0;
}

